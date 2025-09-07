use std::collections::HashMap;
use std::env; // Ortam değişkenlerini okumak için eklendi
use std::fs::File;
use std::io::Read;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ===================================================================================
// ARTIK BİR SABİT YOK. BU DEĞER ÇALIŞMA ZAMANINDA ORTAM DEĞİŞKENİNDEN OKUNACAK.
// ===================================================================================

const WAV_FILE_PATH: &str = "welcome.wav";

struct SipRequest {
    method: String,
    uri: String,
    headers: HashMap<String, String>,
    body: String,
}

fn log(level: &str, log_type: &str, event: &str, message: &str) {
    println!(
        "[{}] [{:<16}] [sentiric-sip-core-service] [{:<5}] {}",
        level, log_type, event, message
    );
}

fn main() -> std::io::Result<()> {
    // DEĞİŞİKLİK: SERVER_IP ortam değişkenini oku.
    // Eğer ayarlanmamışsa, .expect() ile programı panikleterek sonlandır ve kullanıcıya talimat ver.
    let server_ip = env::var("SERVER_IP")
        .expect("Ortam değişkeni bulunamadı: SERVER_IP. Lütfen sunucunun genel IP adresini ayarlayın.");

    let sip_addr = "0.0.0.0:5060";
    let socket = Arc::new(UdpSocket::bind(sip_addr)?);
    
    log("INFO", "BAŞLATILIYOR", "", &format!("SIP sunucusu {} adresinde dinlemede (Public IP: {})...", sip_addr, server_ip));

    let mut buf = [0; 4096];
    loop {
        let (len, remote_addr) = socket.recv_from(&mut buf)?;
        let data = buf[..len].to_vec();
        let sock_clone = Arc::clone(&socket);
        let server_ip_clone = server_ip.clone(); // IP'yi yeni thread için klonla

        thread::spawn(move || {
            // DEĞİŞİKLİK: server_ip'yi fonksiyona parametre olarak geçir.
            if let Err(e) = handle_sip_message(sock_clone, &data, remote_addr, server_ip_clone) {
                log("ERROR", "HATA", "", &format!("İstek işlenirken hata: {}", e));
            }
        });
    }
}

// DEĞİŞİKLİK: Fonksiyon imzasına 'server_ip' parametresi eklendi.
fn handle_sip_message(
    sock: Arc<UdpSocket>,
    data: &[u8],
    remote_addr: SocketAddr,
    server_ip: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_str = std::str::from_utf8(data)?;
    if let Ok(request) = parse_sip_request(request_str) {
        if request.method == "INVITE" {
            // DEĞİŞİKLİK: server_ip'yi bir sonraki fonksiyona da geçir.
            handle_invite(sock, request, remote_addr, server_ip)?;
        }
    }
    Ok(())
}

// DEĞİŞİKLİK: Fonksiyon imzasına 'server_ip' parametresi eklendi.
fn handle_invite(
    sock: Arc<UdpSocket>,
    invite_request: SipRequest,
    remote_addr: SocketAddr,
    server_ip: String, // Artık bir parametre
) -> Result<(), Box<dyn std::error::Error>> {
    let from_user = get_user_from_header(invite_request.headers.get("From").unwrap_or(&String::new()));
    let to_user = get_user_from_header(invite_request.headers.get("To").unwrap_or(&String::new()));
    
    // DEĞİŞİKLİK: Sabit yerine parametreyi kullan.
    let log_details = format!(
        "Arayan: {}, Aranan: {} Kaynak: {} HEDEF: {}:5060",
        from_user, to_user, remote_addr, server_ip
    );
    log("INFO", "ARAMA BAŞLADI", "ÇAĞRI", &log_details);

    let via = invite_request.headers.get("Via").ok_or("Via başlığı yok")?.clone();
    let from = invite_request.headers.get("From").ok_or("From başlığı yok")?.clone();
    let to = invite_request.headers.get("To").ok_or("To başlığı yok")?.clone();
    let call_id = invite_request.headers.get("Call-ID").ok_or("Call-ID başlığı yok")?.clone();
    let cseq = invite_request.headers.get("CSeq").ok_or("CSeq başlığı yok")?.clone();
    let request_uri = invite_request.uri.clone();

    let trying_response = format!(
        "SIP/2.0 100 Trying\r\n\
        Via: {}\r\nFrom: {}\r\nTo: {}\r\nCall-ID: {}\r\nCSeq: {}\r\nContent-Length: 0\r\n\r\n",
        via, from, to, call_id, cseq
    );
    sock.send_to(trying_response.as_bytes(), remote_addr)?;

    let rtp_port = (10000 + (get_timestamp_ms() % 5000) * 2) as u16;
    let rtp_socket = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", rtp_port))?);

    // DEĞİŞİKLİK: Sabit yerine parametreyi kullan.
    let sdp = format!(
        "v=0\r\no=- 0 0 IN IP4 {server_ip}\r\ns=sentiric\r\nc=IN IP4 {server_ip}\r\n\
        t=0 0\r\nm=audio {rtp_port} RTP/AVP 8\r\na=rtpmap:8 PCMA/8000\r\na=sendonly\r\n",
        server_ip = server_ip, // Parametreden gelen değeri kullan
        rtp_port = rtp_port
    );
    
    let to_with_tag = format!("{};tag={:x}", to, get_timestamp_ms());

    // DEĞİŞİKLİK: Sabit yerine parametreyi kullan.
    let ok_response = format!(
        "SIP/2.0 200 OK\r\n\
        Via: {}\r\nFrom: {}\r\nTo: {}\r\nCall-ID: {}\r\nCSeq: {}\r\n\
        Contact: <sip:{}:{}>\r\nContent-Type: application/sdp\r\nContent-Length: {}\r\n\r\n{}",
        via, from, to_with_tag, call_id, cseq, server_ip, 5060, sdp.len(), sdp
    );
    sock.send_to(ok_response.as_bytes(), remote_addr)?;
    log("INFO", "ARAMA CEVAPLANDI", "CEVAP", &log_details);

    let client_rtp_addr = parse_sdp_for_rtp_addr(&invite_request.body)?;
    
    let log_details_clone = log_details.clone();

    thread::spawn(move || {
        log("INFO", "RTP BAŞLADI", "SES", &log_details_clone);
        if let Err(e) = stream_wav_file(rtp_socket, client_rtp_addr) {
            log("ERROR", "RTP HATA", "SES", &format!("RTP akış hatası: {}", e));
        }
        log("INFO", "WAV BİTTİ", "WAV", &log_details_clone);

        let cseq_num: u32 = cseq.split_whitespace().next().unwrap_or("1").parse().unwrap_or(1);
        
        // DEĞİŞİKLİK: Sabit yerine parametreyi kullan. `server_ip` zaten bu scope'a taşındı.
        let bye_request = format!(
            "BYE {} SIP/2.0\r\n\
            Via: SIP/2.0/UDP {}:{};branch=z9hG4bK.{:x}\r\n\
            From: {}\r\nTo: {}\r\nCall-ID: {}\r\nCSeq: {} BYE\r\n\
            Max-Forwards: 70\r\nContent-Length: 0\r\n\r\n",
            request_uri, server_ip, 5060, get_timestamp_ms(), from, to_with_tag, call_id, cseq_num + 1
        );
        if let Err(e) = sock.send_to(bye_request.as_bytes(), remote_addr) {
             log("ERROR", "BYE HATA", "KAPAT", &format!("BYE gönderme hatası: {}", e));
        }
        log("INFO", "ARAMA SONLANDI", "KAPAT", &log_details_clone);
    });

    Ok(())
}

// --- Kodun geri kalanında bir değişiklik yok ---

fn stream_wav_file(
    rtp_socket: Arc<UdpSocket>,
    remote_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let pcm_data = read_wav_file(WAV_FILE_PATH)?;
    let samples_per_packet = 160;
    let mut sequence_number: u16 = (get_timestamp_ms() & 0xFFFF) as u16;
    let mut timestamp: u32 = (get_timestamp_ms() & 0xFFFFFFFF) as u32;
    let ssrc: u32 = (get_timestamp_ms() >> 16) as u32;
    for chunk in pcm_data.chunks(samples_per_packet) {
        let start_time = Instant::now();
        let alaw_payload: Vec<u8> = chunk.iter().map(|&sample| pcm_s16_to_alaw(sample)).collect();
        let mut rtp_packet = vec![0u8; 12 + alaw_payload.len()];
        rtp_packet[0] = 0x80;
        rtp_packet[1] = 0x08;
        rtp_packet[2..4].copy_from_slice(&sequence_number.to_be_bytes());
        rtp_packet[4..8].copy_from_slice(&timestamp.to_be_bytes());
        rtp_packet[8..12].copy_from_slice(&ssrc.to_be_bytes());
        rtp_packet[12..].copy_from_slice(&alaw_payload);
        rtp_socket.send_to(&rtp_packet, remote_addr)?;
        sequence_number = sequence_number.wrapping_add(1);
        timestamp = timestamp.wrapping_add(chunk.len() as u32);
        let elapsed = start_time.elapsed();
        if elapsed < Duration::from_millis(20) {
            thread::sleep(Duration::from_millis(20) - elapsed);
        }
    }
    Ok(())
}

fn parse_sip_request(request_str: &str) -> Result<SipRequest, &'static str> {
    let mut lines = request_str.lines();
    let request_line = lines.next().ok_or("İstek boş")?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or("Metod eksik")?.to_string();
    let uri = parts.next().ok_or("URI eksik")?.to_string();
    let mut headers = HashMap::new();
    for line in lines.by_ref() {
        if line.is_empty() { break; }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    let body = lines.collect::<Vec<&str>>().join("\n");
    Ok(SipRequest { method, uri, headers, body })
}

fn get_user_from_header(header: &str) -> String {
    if let Some(start) = header.find("<sip:") {
        if let Some(end) = header[start..].find('@') {
            return header[start + 5 .. start + end].to_string();
        }
    }
    "bilinmiyor".to_string()
}

fn get_timestamp_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

fn parse_sdp_for_rtp_addr(sdp: &str) -> Result<SocketAddr, &'static str> {
    let mut ip = None;
    let mut port = None;
    for line in sdp.lines() {
        if line.starts_with("c=IN IP4 ") { ip = Some(line.trim_start_matches("c=IN IP4 ").trim()); }
        if line.starts_with("m=audio ") { port = line.split_whitespace().nth(1); }
    }
    match (ip, port) {
        (Some(ip_str), Some(port_str)) => {
            let parsed_ip = ip_str.parse().map_err(|_| "Geçersiz IP adresi")?;
            let parsed_port = port_str.parse().map_err(|_| "Geçersiz port")?;
            Ok(SocketAddr::new(parsed_ip, parsed_port))
        }
        _ => Err("SDP içinde IP veya port bulunamadı"),
    }
}

fn pcm_s16_to_alaw(pcm_val: i16) -> u8 {
    const SIGN_BIT: i16 = 0x80;
    let mut pcm = pcm_val;
    let sign = if pcm < 0 { 0 } else { SIGN_BIT };
    if sign == 0 { pcm = -pcm; }
    if pcm > 32635 { pcm = 32635; }
    let mut seg = 8;
    for i in (0..=7).rev() { if pcm >= (1 << (i + 4)) { seg = i; break; } }
    let aval = if seg < 8 { (seg << 4) | ((pcm >> (seg + 3)) & 0x0F) } else { 0x0F };
    (aval ^ 0x55) as u8
}

fn read_wav_file(path: &str) -> Result<Vec<i16>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut header = [0u8; 44];
    file.read_exact(&mut header)?;
    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" || &header[20..22] != [1, 0] || &header[22..24] != [1, 0] || &header[24..28] != [0x40, 0x1f, 0, 0] || &header[34..36] != [16, 0] {
        return Err("WAV dosyası formatı desteklenmiyor. Lütfen 8000Hz, 16-bit, Mono PCM formatında kullanın.".into());
    }
    let mut pcm_bytes = Vec::new();
    file.read_to_end(&mut pcm_bytes)?;
    Ok(pcm_bytes.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect())
}