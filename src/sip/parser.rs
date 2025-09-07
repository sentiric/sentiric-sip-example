// src/sip/parser.rs

use crate::error::AppError;
use crate::sip::SipRequest;
use std::collections::HashMap;
use std::net::SocketAddr;

// ... parse_sip_request, get_user_from_header, get_uri_from_header, parse_sdp_for_rtp_addr fonksiyonları aynı ...
pub fn parse_sip_request(request_str: &str) -> Result<SipRequest, AppError> { let mut lines = request_str.lines(); let request_line = lines.next().ok_or_else(|| AppError::SipParse("İstek boş".to_string()))?; let mut parts = request_line.split_whitespace(); let method = parts.next().ok_or_else(|| AppError::SipParse("Metod eksik".to_string()))?.to_string(); let _uri = parts.next().ok_or_else(|| AppError::SipParse("URI eksik".to_string()))?; let mut headers = HashMap::new(); for line in lines.by_ref() { if line.is_empty() { break; } if let Some((key, value)) = line.split_once(':') { headers.insert(key.trim().to_lowercase(), value.trim().to_string()); } } let body = lines.collect::<Vec<&str>>().join("\n"); Ok(SipRequest { method, headers, body }) }
pub fn get_user_from_header(header: &str) -> String { if let Some(start) = header.find("<sip:") { if let Some(end) = header[start..].find('@') { return header[start + 5..start + end].to_string(); } } "bilinmiyor".to_string() }
pub fn get_uri_from_header(header: &str) -> Option<String> { header.find('<').and_then(|start| header.find('>').map(|end| header[start + 1..end].to_string())) }
pub fn parse_sdp_for_rtp_addr(sdp: &str) -> Result<SocketAddr, AppError> { let mut ip = None; let mut port = None; for line in sdp.lines() { if line.starts_with("c=IN IP4 ") { ip = Some(line.trim_start_matches("c=IN IP4 ").trim()); } if line.starts_with("m=audio ") { port = line.split_whitespace().nth(1); } } match (ip, port) { (Some(ip_str), Some(port_str)) => Ok(SocketAddr::new(ip_str.parse()?, port_str.parse()?)), _ => Err(AppError::SdpParse("SDP içinde IP adresi veya port bulunamadı".to_string())), } }


// DÜZELTİLDİ: Bu fonksiyon artık anladığı kodekleri değil, m= satırındaki TÜM payload tiplerini String olarak döndürür.
/// Gelen SDP'yi ayrıştırarak istemcinin teklif ettiği tüm payload tiplerinin listesini çıkarır.
pub fn parse_sdp_for_payload_types(sdp: &str) -> Vec<String> {
    let mut payload_types = Vec::new();
    if let Some(audio_line) = sdp.lines().find(|l| l.starts_with("m=audio")) {
        for part in audio_line.split_whitespace().skip(3) {
            // Sadece sayısal payload tiplerini al
            if part.chars().all(char::is_numeric) {
                payload_types.push(part.to_string());
            }
        }
    }
    payload_types
}