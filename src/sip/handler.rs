// src/sip/handler.rs

use super::{
    parser,
    response,
    state::AppState,
    types::{ActiveCall, SipRequest},
};
use crate::{
    config::Config,
    error::AppError,
    rtp::{self, codec::Codec}, // DÜZELTİLDİ: Tek ve doğru 'use' satırı
    util,
};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

/// Gelen SIP mesajını ayrıştırır ve ilgili metoda göre yönlendirir.
pub fn handle_sip_message(start_time: Instant, sock: Arc<UdpSocket>, data: &[u8], remote_addr: SocketAddr, config: Arc<Config>, state: AppState) -> Result<(), AppError> {
    let request_str = std::str::from_utf8(data)?;
    match parser::parse_sip_request(request_str) {
        Ok(request) => match request.method.as_str() {
            "INVITE" => handle_invite(start_time, sock, request, remote_addr, config, state),
            "ACK" => {
                util::log(start_time, "DEBUG", "TEYİT ALINDI", "SIP", &format!("İstemciden ACK alındı: {}", remote_addr));
                Ok(())
            }
            "BYE" => handle_bye(start_time, request, state),
            _ => Ok(()), // Diğer metodları şimdilik yoksay
        },
        Err(e) => {
            util::log(start_time, "WARN", "PARSE HATASI", "SIP", &format!("Gelen SIP mesajı ayrıştırılamadı (Kaynak: {}): {}", remote_addr, e));
            Err(e)
        }
    }
}

/// Gelen INVITE isteğini işler, aramayı cevaplar ve RTP akışını başlatır.
fn handle_invite(start_time: Instant, sock: Arc<UdpSocket>, request: SipRequest, remote_addr: SocketAddr, config: Arc<Config>, state: AppState) -> Result<(), AppError> {
    let call_id = request.headers.get("call-id").ok_or_else(|| AppError::MissingHeader("Call-ID".to_string()))?.clone();
    let from_header = request.headers.get("from").ok_or_else(|| AppError::MissingHeader("From".to_string()))?.clone();
    let to_header = request.headers.get("to").ok_or_else(|| AppError::MissingHeader("To".to_string()))?.clone();
    let contact_header = request.headers.get("contact").ok_or_else(|| AppError::MissingHeader("Contact".to_string()))?.clone();
    let contact_uri = parser::get_uri_from_header(&contact_header).ok_or_else(|| AppError::SipParse("Contact URI bulunamadı".to_string()))?;
    let record_route_header = request.headers.get("record-route").cloned();

    let log_details = format!("Arayan: {}, Aranan: {} Kaynak: {}", parser::get_user_from_header(&from_header), parser::get_user_from_header(&to_header), remote_addr);
    util::log(start_time, "INFO", "ARAMA BAŞLADI", "ÇAĞRI", &log_details);

    sock.send_to(response::build_trying_response(&request).as_bytes(), remote_addr)?;

    let client_codecs = parser::parse_sdp_for_codecs(&request.body);
    const SUPPORTED_CODECS_IN_ORDER: [Codec; 3] = [Codec::G729, Codec::PCMA, Codec::PCMU];
    
    let chosen_codec = SUPPORTED_CODECS_IN_ORDER.iter()
        .find(|&supported_codec| client_codecs.contains(supported_codec))
        .copied()
        .ok_or(AppError::UnsupportedCodec)?;

    util::log(start_time, "DEBUG", "CODEC ANLAŞMASI", "SIP", &format!("İstemci desteklenen kodekler: {:?}. Seçilen: {:?}", client_codecs, chosen_codec));

    let rtp_port = (10000 + (util::get_timestamp_ms() % 5000) * 2) as u16;
    let rtp_socket = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", rtp_port))?);
    let to_tag = format!("{:x}", util::get_timestamp_ms());

    let ok_response = response::build_ok_response(&request, &config, rtp_port, &to_tag, chosen_codec);
    sock.send_to(ok_response.as_bytes(), remote_addr)?;
    util::log(start_time, "INFO", "ARAMA CEVAPLANDI", "CEVAP", &format!("200 OK -> {}. RTP port: {}. Codec: {:?}", remote_addr, rtp_port, chosen_codec));

    let client_rtp_addr = parser::parse_sdp_for_rtp_addr(&request.body)?;

    let call = ActiveCall {
        remote_addr,
        from_header: from_header.clone(),
        to_header: format!("{};tag={}", to_header, to_tag),
        call_id: call_id.clone(),
        contact_uri,
        record_route_header,
    };
    state.lock().unwrap().insert(call_id.clone(), call);

    let cseq_header = request.headers.get("cseq").ok_or_else(|| AppError::MissingHeader("CSeq".to_string()))?.clone();
    let wav_path_clone = config.wav_file_path.clone();

    thread::spawn(move || {
        if let Err(e) = rtp::stream::stream_wav_file(start_time, rtp_socket, client_rtp_addr, &wav_path_clone, chosen_codec) {
            util::log(start_time, "ERROR", "RTP HATASI", "SES", &e.to_string());
        }
        util::log(start_time, "INFO", "WAV BİTTİ", "WAV", &log_details);

        if let Some(call) = state.lock().unwrap().remove(&call_id) {
            util::log(start_time, "INFO", "ARAMA SONLANDIRILIYOR", "KAPAT", "Ses dosyası bitti, sunucu tarafından kapatılıyor...");
            let cseq_num = cseq_header.split_whitespace().next().unwrap_or("1").parse().unwrap_or(1) + 10;
            let bye_request = response::build_bye_request(&call, &config, cseq_num);
            if let Err(e) = sock.send_to(bye_request.as_bytes(), call.remote_addr) {
                util::log(start_time, "ERROR", "BYE GÖNDERME HATASI", "KAPAT", &e.to_string());
            }
        }
    });

    Ok(())
}

/// İstemciden gelen BYE isteğini işler.
fn handle_bye(start_time: Instant, request: SipRequest, state: AppState) -> Result<(), AppError> {
    if let Some(call_id) = request.headers.get("call-id") {
        if state.lock().unwrap().remove(call_id).is_some() {
            util::log(start_time, "INFO", "ARAMA SONLANDI", "KAPAT", &format!("İstemci tarafından kapatıldı (Call-ID: {})", call_id));
        }
    }
    Ok(())
}