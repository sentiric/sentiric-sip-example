use std::fmt;

/// Uygulama genelinde kullanılacak olan merkezi hata türü.
#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    AddrParse(std::net::AddrParseError),
    NumParse(std::num::ParseIntError),
    SipParse(String),
    SdpParse(String),
    MissingHeader(String),
    UnsupportedCodec,
    RtpSendError(String),
    WavFileError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "G/Ç Hatası: {}", e),
            AppError::Utf8(e) => write!(f, "Geçersiz UTF-8 dizisi: {}", e),
            AppError::AddrParse(e) => write!(f, "Adres ayrıştırma hatası: {}", e),
            AppError::NumParse(e) => write!(f, "Sayı ayrıştırma hatası: {}", e),
            AppError::SipParse(s) => write!(f, "SIP ayrıştırma hatası: {}", s),
            AppError::SdpParse(s) => write!(f, "SDP ayrıştırma hatası: {}", s),
            AppError::MissingHeader(s) => write!(f, "Gerekli başlık eksik: {}", s),
            AppError::UnsupportedCodec => write!(f, "Desteklenen bir ses codec'i bulunamadı (PCMU/PCMA gerekli)"),
            AppError::RtpSendError(s) => write!(f, "RTP gönderme hatası: {}", s),
            AppError::WavFileError(s) => write!(f, "WAV dosyası hatası: {}", s),
        }
    }
}

impl std::error::Error for AppError {}

macro_rules! impl_from_error {
    ($from:ty, $to:ident) => {
        impl From<$from> for AppError {
            fn from(e: $from) -> Self {
                AppError::$to(e)
            }
        }
    };
}

impl_from_error!(std::io::Error, Io);
impl_from_error!(std::str::Utf8Error, Utf8);
impl_from_error!(std::net::AddrParseError, AddrParse);
impl_from_error!(std::num::ParseIntError, NumParse);