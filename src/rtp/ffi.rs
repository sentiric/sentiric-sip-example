// src/rtp/ffi.rs

use std::ffi::{c_short, c_uchar, c_void}; // DÜZELTİLDİ: Kullanılmayan 'c_int' kaldırıldı.

#[link(name = "g729", kind = "static")]
extern "C" {
    // C header dosyasındaki orijinal isim: bcg729EncoderChannelContextStruct *initBcg729EncoderChannel(uint8_t enableVAD);
    pub fn initBcg729EncoderChannel(enableVAD: c_uchar) -> *mut c_void;

    // C header dosyasındaki orijinal isim: void closeBcg729EncoderChannel(bcg729EncoderChannelContextStruct *encoderChannelContext);
    pub fn closeBcg729EncoderChannel(encoderChannelContext: *mut c_void);

    // C header dosyasındaki orijinal isim: void bcg729Encoder(..., const int16_t inputFrame[], uint8_t bitStream[], uint8_t *bitStreamLength);
    pub fn bcg729Encoder(
        encoderChannelContext: *mut c_void,
        inputFrame: *const c_short,
        bitStream: *mut c_uchar,
        bitStreamLength: *mut c_uchar,
    );
}