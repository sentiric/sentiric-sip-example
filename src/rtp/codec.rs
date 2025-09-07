// src/rtp/codec.rs

use super::ffi;
use std::ffi::c_void;

/// Desteklenen ses formatlarını (codec) temsil eden enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // Eq eklendi
pub enum Codec {
    G729, // Payload Type 18
    PCMA, // G.711 A-Law (Payload Type 8)
    PCMU, // G.711 µ-Law (Payload Type 0)
}

// ... pcm_s16_to_alaw ve pcm_s16_to_ulaw fonksiyonları aynı kalır ...
pub fn pcm_s16_to_alaw(pcm_val: i16) -> u8 { let mut pval = pcm_val; let sign = (pval & 0x8000u16 as i16) != 0; if sign { pval = !pval; } if pval < 256 { pval >>= 4; } else if pval < 512 { pval = (pval >> 5) + 16; } else if pval < 1024 { pval = (pval >> 6) + 32; } else if pval < 2048 { pval = (pval >> 7) + 48; } else if pval < 4096 { pval = (pval >> 8) + 64; } else if pval < 8192 { pval = (pval >> 9) + 80; } else if pval < 16384 { pval = (pval >> 10) + 96; } else { pval = (pval >> 11) + 112; } let result = if sign { pval as u8 } else { pval as u8 | 0x80 }; result ^ 0x55 }
pub fn pcm_s16_to_ulaw(pcm_val: i16) -> u8 { const BIAS: i16 = 0x84; const MAX: i16 = 32100; let sign = (pcm_val >> 8) & 0x80; let mut sample = if sign != 0 { -pcm_val } else { pcm_val }; if sample > MAX { sample = MAX; } sample += BIAS; let mut exponent: i16 = 7; for i in (0..=6).rev() { if sample <= ((1 << (i + 5)) - 1) { exponent = i; break; } } let mantissa = (sample >> (exponent + 3)) & 0x0F; let ulaw: i16 = !((exponent << 4) | mantissa); if sign != 0 { (ulaw & 0x7F) as u8 } else { (ulaw | 0x80) as u8 } }

// DÜZELTİLDİ: Bu fonksiyon artık hafıza (state) yönetimini yapmıyor.
// Sadece kendisine verilen hafızayı kullanarak tek bir kodlama işlemi gerçekleştiriyor.
pub fn pcm_s16_to_g729(encoder_state: *mut c_void, pcm_samples: &[i16]) -> Vec<u8> {
    if pcm_samples.len() != 80 || encoder_state.is_null() {
        return Vec::new();
    }

    unsafe {
        let mut output_buffer = vec![0u8; 10];
        let mut output_len: u8 = 0;

        ffi::bcg729Encoder(
            encoder_state,
            pcm_samples.as_ptr(),
            output_buffer.as_mut_ptr(),
            &mut output_len,
        );
        
        output_buffer.truncate(output_len as usize);
        output_buffer
    }
}