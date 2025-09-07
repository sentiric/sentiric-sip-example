// src/sip/response.rs

use crate::config::Config;
use crate::rtp::codec::Codec;
use crate::sip::{types::ActiveCall, SipRequest};
use crate::util;

pub fn build_trying_response(req: &SipRequest) -> String {
    format!(
        "SIP/2.0 100 Trying\r\nVia: {}\r\nFrom: {}\r\nTo: {}\r\nCall-ID: {}\r\nCSeq: {}\r\nContent-Length: 0\r\n\r\n",
        req.headers.get("via").unwrap_or(&"".to_string()),
        req.headers.get("from").unwrap_or(&"".to_string()),
        req.headers.get("to").unwrap_or(&"".to_string()),
        req.headers.get("call-id").unwrap_or(&"".to_string()),
        req.headers.get("cseq").unwrap_or(&"".to_string())
    )
}

pub fn build_ok_response(req: &SipRequest, config: &Config, rtp_port: u16, to_tag: &str, codec: Codec) -> String {
    // DÜZELTİLDİ: match bloğuna G.729 seçeneği eklendi.
    let (payload_type, codec_name) = match codec {
        Codec::G729 => (18, "G729"),
        Codec::PCMU => (0, "PCMU"),
        Codec::PCMA => (8, "PCMA"),
    };

    let sdp = format!(
        "v=0\r\no=- 0 0 IN IP4 {ip}\r\ns=sentiric\r\nc=IN IP4 {ip}\r\nt=0 0\r\nm=audio {port} RTP/AVP {pt}\r\na=rtpmap:{pt} {name}/8000\r\na=sendonly\r\n",
        ip = config.server_ip, port = rtp_port, pt = payload_type, name = codec_name
    );

    format!(
        "SIP/2.0 200 OK\r\nVia: {}\r\nFrom: {}\r\nTo: {};tag={}\r\nCall-ID: {}\r\nCSeq: {}\r\nContact: <sip:{}:{}>\r\nContent-Type: application/sdp\r\nContent-Length: {}\r\n\r\n{}",
        req.headers.get("via").unwrap_or(&"".to_string()),
        req.headers.get("from").unwrap_or(&"".to_string()),
        req.headers.get("to").unwrap_or(&"".to_string()), to_tag,
        req.headers.get("call-id").unwrap_or(&"".to_string()),
        req.headers.get("cseq").unwrap_or(&"".to_string()),
        config.server_ip, config.sip_port,
        sdp.len(), sdp
    )
}

pub fn build_bye_request(call: &ActiveCall, config: &Config, cseq_num: u32) -> String {
    let our_from_header = call.to_header.replace(super::parser::get_user_from_header(&call.to_header).as_str(), "sentiric");

    let record_route_line = call.record_route_header
        .as_ref()
        .map_or("".to_string(), |rr| format!("Record-Route: {}\r\n", rr));

    format!(
        "BYE {} SIP/2.0\r\nVia: SIP/2.0/UDP {}:{};branch=z9hG4bK.{:x}\r\n{}From: {}\r\nTo: {}\r\nCall-ID: {}\r\nCSeq: {} BYE\r\nMax-Forwards: 70\r\nContent-Length: 0\r\n\r\n",
        call.contact_uri,
        config.server_ip, config.sip_port, util::get_timestamp_ms(),
        record_route_line,
        our_from_header,
        call.from_header,
        call.call_id,
        cseq_num
    )
}