// src/sip/types.rs
use std::collections::HashMap;
use std::net::SocketAddr;

pub struct ActiveCall {
    pub remote_addr: SocketAddr,
    pub from_header: String,
    pub to_header: String,
    pub call_id: String,
    pub contact_uri: String,
    pub record_route_header: Option<String>, // Bu alan mevcut olmalı
}

pub struct SipRequest {
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}