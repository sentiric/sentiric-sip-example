// ### File: src/sip/types.rs

use std::collections::HashMap;
use std::net::SocketAddr;

pub struct ActiveCall {
    pub remote_addr: SocketAddr,
    pub from_header: String,
    pub to_header: String,
    pub call_id: String,
    pub contact_uri: String,
    pub record_route_header: Option<String>,
}

pub struct SipRequest {
    pub method: String,
    // --- GÜNCELLEME BAŞLANGICI ---
    // 'Via' başlıkları birden fazla olabileceğinden, onları ayrı bir vektörde tutuyoruz.
    // RFC 3261'e göre, yanıtlarda tüm Via başlıkları sırasıyla geri gönderilmelidir.
    pub via_headers: Vec<String>,
    // --- GÜNCELLEME SONU ---
    pub headers: HashMap<String, String>,
    pub body: String,
}