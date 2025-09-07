// src/rtp/stream.rs

use super::{
    codec::{self, Codec},
    wav,
};
use crate::{error::AppError, util};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub fn stream_wav_file(
    start_time: Instant,
    rtp_socket: Arc<UdpSocket>,
    remote_addr: SocketAddr,
    wav_path: &str,
    chosen_codec: Codec,
) -> Result<(), AppError> {
    let pcm_data = wav::read_wav_file(wav_path)?;
    let (samples_per_packet, packet_interval, payload_type) = match chosen_codec {
        Codec::PCMU | Codec::PCMA => (160, Duration::from_millis(20), if chosen_codec == Codec::PCMU { 0 } else { 8 }),
        Codec::G729 => (160, Duration::from_millis(20), 18),
    };

    let mut sequence_number: u16 = (util::get_timestamp_ms() & 0xFFFF) as u16;
    let mut timestamp: u32 = (util::get_timestamp_ms() & 0xFFFFFFFF) as u32;
    let ssrc: u32 = (util::get_timestamp_ms() >> 16) as u32;
    let mut next_packet_time = Instant::now();

    util::log(start_time, "INFO", "RTP BAŞLADI", "SES", &format!("Ses akışı başlıyor -> {}", remote_addr));

    // --- DÜZELTİLDİ: G.729 Durum Yönetimi ---
    // G.729 Kodlayıcının "hafızasını" döngüden ÖNCE SADECE BİR KEZ oluşturuyoruz.
    let g729_encoder_state = if chosen_codec == Codec::G729 {
        unsafe { Some(super::ffi::initBcg729EncoderChannel(0)) } // 0 = VAD kapalı
    } else {
        None
    };

    if chosen_codec == Codec::G729 && g729_encoder_state.unwrap().is_null() {
        return Err(AppError::RtpSendError("G.729 encoder state oluşturulamadı.".to_string()));
    }
    // --- BİTTİ ---

    for chunk in pcm_data.chunks(samples_per_packet) {
        if chunk.len() != samples_per_packet { continue; }

        let payload: Vec<u8> = match chosen_codec {
            Codec::PCMU => chunk.iter().map(|&s| codec::pcm_s16_to_ulaw(s)).collect(),
            Codec::PCMA => chunk.iter().map(|&s| codec::pcm_s16_to_alaw(s)).collect(),
            Codec::G729 => {
                // Her seferinde hafızayı SIFIRLAMAK YERİNE, mevcut hafızayı kullanarak kodlama yapıyoruz.
                let mut first_frame = codec::pcm_s16_to_g729(g729_encoder_state.unwrap(), &chunk[0..80]);
                let second_frame = codec::pcm_s16_to_g729(g729_encoder_state.unwrap(), &chunk[80..160]);
                first_frame.extend(second_frame);
                first_frame
            }
        };

        let mut rtp_packet = vec![0u8; 12 + payload.len()];
        rtp_packet[0] = 0x80;
        rtp_packet[1] = payload_type;
        rtp_packet[2..4].copy_from_slice(&sequence_number.to_be_bytes());
        rtp_packet[4..8].copy_from_slice(&timestamp.to_be_bytes());
        rtp_packet[8..12].copy_from_slice(&ssrc.to_be_bytes());
        rtp_packet[12..].copy_from_slice(&payload);

        rtp_socket.send_to(&rtp_packet, remote_addr)
            .map_err(|_| AppError::RtpSendError(format!("RTP paketi gönderilemedi: {}", remote_addr)))?;

        sequence_number = sequence_number.wrapping_add(1);
        timestamp = timestamp.wrapping_add(chunk.len() as u32);
        next_packet_time += packet_interval;
        let sleep_duration = next_packet_time.saturating_duration_since(Instant::now());
        thread::sleep(sleep_duration);
    }

    // --- DÜZELTİLDİ: G.729 Durum Yönetimi ---
    // Döngü BİTTİKTEN SONRA hafızayı SADECE BİR KEZ temizliyoruz.
    if let Some(state) = g729_encoder_state {
        if !state.is_null() {
            unsafe { super::ffi::closeBcg729EncoderChannel(state); }
        }
    }
    // --- BİTTİ ---

    Ok(())
}