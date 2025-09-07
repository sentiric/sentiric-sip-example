use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// --- Uygulama Çapında Paylaşılan Yapılar ---

/// Paylaşılan uygulama durumu. Aktif aramaları thread'ler arası güvenli bir şekilde tutar.
type AppState = Arc<Mutex<HashMap<String, ActiveCall>>>;

/// Aktif bir aramanın oturum bilgilerini içeren yapı.
struct ActiveCall {
    remote_addr: SocketAddr,
    from_header: String,
    to_header: String,
    call_id: String,
    contact_uri: String,
}

/// Gelen bir SIP isteğini temsil eden yapı.
struct SipRequest {
    method: String,
    headers: HashMap<String, String>,
    body: String,
}

/// Desteklenen ses formatlarını (codec) temsil eden enum.
/// Bu, kod içinde `0` veya `8` gibi "sihirli sayılar" kullanmak yerine
/// `Codec::PCMU` gibi anlaşılır ifadeler kullanmamızı sağlar.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Codec {
    PCMU, // G.711 µ-Law (Payload Type 0)
    PCMA, // G.711 A-Law (Payload Type 8)
}

// --- Ana Uygulama Mantığı ---

fn main() -> std::io::Result<()> {
    let start_time = Instant::now();

    // Yapılandırmayı ortam değişkenlerinden oku, yoksa varsayılan değerleri kullan.
    let server_ip = env::var("SERVER_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let sip_port = env::var("SIP_PORT").unwrap_or_else(|_| "5060".to_string());
    let wav_file_path = env::var("WAV_FILE").unwrap_or_else(|_| "welcome.wav".to_string());
    
    let active_calls: AppState = Arc::new(Mutex::new(HashMap::new()));
    let sip_addr = format!("0.0.0.0:{}", sip_port);
    let socket = Arc::new(UdpSocket::bind(&sip_addr)?);
    
    log(start_time, "INFO", "BAŞLATILIYOR", "", &format!("SIP sunucusu {} adresinde dinlemede (Public IP: {}, WAV: {})...", sip_addr, server_ip, wav_file_path));

    let mut buf = [0; 4096];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, remote_addr)) => {
                let data = buf[..len].to_vec();
                let sock_clone = Arc::clone(&socket);
                let server_ip_clone = server_ip.clone();
                let wav_path_clone = wav_file_path.clone();
                let state_clone = Arc::clone(&active_calls);
                thread::spawn(move || {
                    if let Err(e) = handle_sip_message(start_time, sock_clone, &data, remote_addr, server_ip_clone, wav_path_clone, state_clone) {
                        log(start_time, "ERROR", "İŞLEM HATASI", "", &format!("Gelen istek işlenemedi (Kaynak: {}): {}", remote_addr, e));
                    }
                });
            },
            Err(e) => { log(start_time, "ERROR", "SOKET HATASI", "", &format!("UDP soketinden veri okunamadı: {}", e)); }
        }
    }
}

/// Standart formatta loglama yapan fonksiyon. Bağımlılık kullanmaz.
/// Logun başına, programın başlangıcından itibaren geçen süreyi saniye cinsinden ekler.
fn log(start_time: Instant, level: &str, log_type: &str, event: &str, message: &str) {
    let elapsed = start_time.elapsed();
    println!("[{:09.3}] [{}] [{:<16}] [sentiric] [{:<5}] {}", elapsed.as_secs_f32(), level, log_type, event, message);
}

/// Gelen SIP mesajını ayrıştırır ve ilgili metoda göre yönlendirir.
fn handle_sip_message(start_time: Instant, sock: Arc<UdpSocket>, data: &[u8], remote_addr: SocketAddr, server_ip: String, wav_path: String, state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let request_str = std::str::from_utf8(data).map_err(|_| "Geçersiz UTF-8")?;
    match parse_sip_request(request_str) {
        Ok(request) => {
            match request.method.as_str() {
                "INVITE" => handle_invite(start_time, sock, request, remote_addr, server_ip, wav_path, state)?,
                "ACK" => log(start_time, "DEBUG", "TEYİT ALINDI", "SIP", &format!("İstemciden ACK alındı: {}", remote_addr)),
                "BYE" => handle_bye(start_time, request, state)?,
                _ => {} // Bilinmeyen metodları yoksay
            }
        },
        Err(e) => { log(start_time, "WARN", "PARSE HATASI", "SIP", &format!("Gelen SIP mesajı ayrıştırılamadı (Kaynak: {}): {}", remote_addr, e)); }
    }
    Ok(())
}

/// Gelen INVITE isteğini işler, codec anlaşması yapar, aramayı cevaplar ve RTP akışını başlatır.
fn handle_invite(start_time: Instant, sock: Arc<UdpSocket>, invite_request: SipRequest, remote_addr: SocketAddr, server_ip: String, wav_path: String, state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let call_id = invite_request.headers.get("call-id").ok_or("Call-ID başlığı yok")?.clone();
    let from_header = invite_request.headers.get("from").ok_or("From başlığı yok")?.clone();
    let to_header = invite_request.headers.get("to").ok_or("To başlığı yok")?.clone();
    let via_header = invite_request.headers.get("via").ok_or("Via başlığı yok")?.clone();
    let cseq_header = invite_request.headers.get("cseq").ok_or("CSeq başlığı yok")?.clone();
    let contact_header = invite_request.headers.get("contact").ok_or("Contact başlığı yok")?.clone();
    let contact_uri = get_uri_from_header(&contact_header).ok_or("Contact URI bulunamadı")?;

    let log_details = format!("Arayan: {}, Aranan: {} Kaynak: {}", get_user_from_header(&from_header), get_user_from_header(&to_header), remote_addr);
    log(start_time, "INFO", "ARAMA BAŞLADI", "ÇAĞRI", &log_details);
    
    // --- Codec Anlaşması (Negotiation) ---
    let client_codecs = parse_sdp_for_codecs(&invite_request.body);
    let chosen_codec = match client_codecs.iter().find(|&&c| c == Codec::PCMU) {
        Some(c) => *c,
        None => match client_codecs.iter().find(|&&c| c == Codec::PCMA) {
            Some(c) => *c,
            None => {
                log(start_time, "ERROR", "CODEC UYUMSUZ", "SIP", "İstemci PCMU veya PCMA desteklemiyor.");
                return Err("Uyumlu codec bulunamadı".into());
            }
        }
    };
    log(start_time, "DEBUG", "CODEC ANLAŞMASI", "SIP", &format!("İstemci desteklenen kodekler: {:?}. Seçilen: {:?}", client_codecs, chosen_codec));

    let trying_response = format!("SIP/2.0 100 Trying\r\nVia: {}\r\nFrom: {}\r\nTo: {}\r\nCall-ID: {}\r\nCSeq: {}\r\nContent-Length: 0\r\n\r\n", via_header, from_header, to_header, call_id, cseq_header);
    sock.send_to(trying_response.as_bytes(), remote_addr)?;

    let rtp_port = (10000 + (get_timestamp_ms() % 5000) * 2) as u16;
    let rtp_socket = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", rtp_port))?);
    let to_tag = format!("{:x}", get_timestamp_ms());
    let to_with_tag = format!("{};tag={}", to_header, to_tag);
    
    // Kendi SDP'mizi, anlaşılan codec'e göre oluştur.
    let (payload_type, codec_name) = match chosen_codec {
        Codec::PCMU => (0, "PCMU"),
        Codec::PCMA => (8, "PCMA"),
    };
    let sdp = format!(
        "v=0\r\no=- 0 0 IN IP4 {ip}\r\ns=sentiric\r\nc=IN IP4 {ip}\r\nt=0 0\r\nm=audio {port} RTP/AVP {pt}\r\na=rtpmap:{pt} {name}/8000\r\na=sendonly\r\n",
        ip = server_ip,
        port = rtp_port,
        pt = payload_type,
        name = codec_name
    );
    
    let ok_response = format!(
        "SIP/2.0 200 OK\r\nVia: {}\r\nFrom: {}\r\nTo: {}\r\nCall-ID: {}\r\nCSeq: {}\r\nContact: <sip:{}:{}>\r\nContent-Type: application/sdp\r\nContent-Length: {}\r\n\r\n{}",
        via_header, from_header, to_with_tag, call_id, cseq_header, server_ip, 5060, sdp.len(), sdp
    );
    sock.send_to(ok_response.as_bytes(), remote_addr)?;
    log(start_time, "INFO", "ARAMA CEVAPLANDI", "CEVAP", &format!("200 OK -> {}. RTP port: {}. Codec: {}", remote_addr, rtp_port, codec_name));

    let client_rtp_addr = parse_sdp_for_rtp_addr(&invite_request.body)?;
    
    state.lock().unwrap().insert(call_id.clone(), ActiveCall {
        remote_addr,
        from_header: from_header.clone(),
        to_header: to_with_tag.clone(),
        call_id: call_id.clone(),
        contact_uri,
    });

    thread::spawn(move || {
        if let Err(e) = stream_wav_file(start_time, rtp_socket, client_rtp_addr, &wav_path, chosen_codec) {
            log(start_time, "ERROR", "RTP HATASI", "SES", &e.to_string());
        }
        log(start_time, "INFO", "WAV BİTTİ", "WAV", &log_details);

        if let Some(call) = state.lock().unwrap().remove(&call_id) {
            log(start_time, "INFO", "ARAMA SONLANDIRILIYOR", "KAPAT", "Ses dosyası bitti, sunucu tarafından kapatılıyor...");
            let cseq_num: u32 = cseq_header.split_whitespace().next().unwrap().parse().unwrap_or(1);
            let our_from_header = call.to_header.replace(get_user_from_header(&call.to_header).as_str(), "sentiric");
            let bye_request = format!(
                "BYE {} SIP/2.0\r\nVia: SIP/2.0/UDP {}:{};branch=z9hG4bK.{:x}\r\nFrom: {}\r\nTo: {}\r\nCall-ID: {}\r\nCSeq: {} BYE\r\nMax-Forwards: 70\r\nContent-Length: 0\r\n\r\n",
                call.contact_uri, server_ip, 5060, get_timestamp_ms(), our_from_header, call.from_header, call.call_id, cseq_num + 10
            );
            if let Err(e) = sock.send_to(bye_request.as_bytes(), call.remote_addr) {
                log(start_time, "ERROR", "BYE GÖNDERME HATASI", "KAPAT", &e.to_string());
            }
        }
    });

    Ok(())
}

/// İstemciden gelen BYE isteğini işler.
fn handle_bye(start_time: Instant, request: SipRequest, state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(call_id) = request.headers.get("call-id") {
        if state.lock().unwrap().remove(call_id).is_some() {
            log(start_time, "INFO", "ARAMA SONLANDI", "KAPAT", &format!("İstemci tarafından kapatıldı (Call-ID: {})", call_id));
        }
    }
    Ok(())
}

/// WAV dosyasını okur ve anlaşılan codec ile hassas zamanlama yaparak RTP paketleri olarak gönderir.
fn stream_wav_file(start_time: Instant, rtp_socket: Arc<UdpSocket>, remote_addr: SocketAddr, wav_path: &str, codec: Codec) -> Result<(), Box<dyn std::error::Error>> {
    let pcm_data = read_wav_file(wav_path)?;
    let samples_per_packet = 160;
    let packet_interval = Duration::from_millis(20);
    let mut sequence_number: u16 = (get_timestamp_ms() & 0xFFFF) as u16;
    let mut timestamp: u32 = (get_timestamp_ms() & 0xFFFFFFFF) as u32;
    let ssrc: u32 = (get_timestamp_ms() >> 16) as u32;
    let mut next_packet_time = Instant::now();
    log(start_time, "INFO", "RTP BAŞLADI", "SES", &format!("Ses akışı başlıyor -> {}", remote_addr));

    let (payload_type, encoder): (u8, fn(i16) -> u8) = match codec {
        Codec::PCMU => (0, pcm_s16_to_ulaw),
        Codec::PCMA => (8, pcm_s16_to_alaw),
    };

    for chunk in pcm_data.chunks(samples_per_packet) {
        let payload: Vec<u8> = chunk.iter().map(|&sample| encoder(sample)).collect();
        let mut rtp_packet = vec![0u8; 12 + payload.len()];
        rtp_packet[0] = 0x80;
        rtp_packet[1] = payload_type;
        rtp_packet[2..4].copy_from_slice(&sequence_number.to_be_bytes());
        rtp_packet[4..8].copy_from_slice(&timestamp.to_be_bytes());
        rtp_packet[8..12].copy_from_slice(&ssrc.to_be_bytes());
        rtp_packet[12..].copy_from_slice(&payload);
        if rtp_socket.send_to(&rtp_packet, remote_addr).is_err() {
            return Err(format!("RTP paketi gönderilemedi: {}", remote_addr).into());
        }
        sequence_number = sequence_number.wrapping_add(1);
        timestamp = timestamp.wrapping_add(chunk.len() as u32);
        next_packet_time += packet_interval;
        let sleep_duration = next_packet_time.saturating_duration_since(Instant::now());
        thread::sleep(sleep_duration);
    }
    Ok(())
}

// --- Yardımcı Fonksiyonlar (SIP, SDP, Dosya Okuma) ---

fn parse_sip_request(request_str: &str) -> Result<SipRequest, &'static str> {
    let mut lines = request_str.lines();
    let request_line = lines.next().ok_or("İstek boş")?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or("Metod eksik")?.to_string();
    let _uri = parts.next().ok_or("URI eksik")?;
    let mut headers = HashMap::new();
    for line in lines.by_ref() {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    let body = lines.collect::<Vec<&str>>().join("\n");
    Ok(SipRequest { method, headers, body })
}

fn get_user_from_header(header: &str) -> String {
    if let Some(start) = header.find("<sip:") {
        if let Some(end) = header[start..].find('@') {
            return header[start + 5..start + end].to_string();
        }
    }
    "bilinmiyor".to_string()
}

fn get_uri_from_header(header: &str) -> Option<String> {
    header.find('<').and_then(|start| header.find('>').map(|end| header[start + 1..end].to_string()))
}

fn get_timestamp_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

fn parse_sdp_for_rtp_addr(sdp: &str) -> Result<SocketAddr, &'static str> {
    let mut ip = None;
    let mut port = None;
    for line in sdp.lines() {
        if line.starts_with("c=IN IP4 ") {
            ip = Some(line.trim_start_matches("c=IN IP4 ").trim());
        }
        if line.starts_with("m=audio ") {
            port = line.split_whitespace().nth(1);
        }
    }
    match (ip, port) {
        (Some(ip_str), Some(port_str)) => Ok(SocketAddr::new(
            ip_str.parse().map_err(|_| "Geçersiz IP")?,
            port_str.parse().map_err(|_| "Geçersiz port")?,
        )),
        _ => Err("SDP'de IP/port yok"),
    }
}

fn read_wav_file(path: &str) -> Result<Vec<i16>, Box<dyn std::error::Error>> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("'{}' dosyası açılamadı: {}", path, e).into()),
    };
    let mut header = [0u8; 44];
    file.read_exact(&mut header)?;
    if &header[0..4] != b"RIFF"
        || &header[8..12] != b"WAVE"
        || &header[20..22] != [1, 0]
        || &header[22..24] != [1, 0]
        || &header[24..28] != [0x40, 0x1f, 0, 0]
        || &header[34..36] != [16, 0]
    {
        return Err(format!("'{}' dosyası formatı desteklenmiyor. Lütfen 8000Hz, 16-bit, Mono PCM formatında kullanın.", path).into());
    }
    let mut pcm_bytes = Vec::new();
    file.read_to_end(&mut pcm_bytes)?;
    Ok(pcm_bytes.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect())
}

/// Gelen SDP'yi ayrıştırarak istemcinin desteklediği codec listesini çıkarır.
fn parse_sdp_for_codecs(sdp: &str) -> Vec<Codec> {
    let mut codecs = Vec::new();
    for line in sdp.lines() {
        if line.starts_with("m=audio") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for &part in &parts[3..] {
                match part.trim_end_matches(|c: char| !c.is_digit(10)) {
                    "0" => codecs.push(Codec::PCMU),
                    "8" => codecs.push(Codec::PCMA),
                    _ => {}
                }
            }
        }
    }
    codecs
}

// ===================================================================================
// ========================== G.711 SES CODEC'LERİ ===================================
// ===================================================================================

/// Bu, ITU-T G.711 standardına uygun, endüstri standardı bir A-Law çevrim algoritmasıdır.
fn pcm_s16_to_alaw(pcm_val: i16) -> u8 {
    let mut sample = pcm_val;
    
    // İşaret bitini kontrol et. `i16` negatifse, en soldaki bit 1'dir.
    // Rust'ın katı tür denetimi nedeniyle, bu işlemi `u16` üzerinde yapıp sonra `i16`'ya çeviriyoruz.
    let sign = (sample & (0x8000u16 as i16)) != 0;

    if sign {
        // Negatifse, mutlak değerini al.
        sample = -sample;
    }

    if sample > 0x7FFF {
        sample = 0x7FFF;
    }
    
    let mut exponent: i16 = 7;
    for i in (0..=6).rev() {
        if (sample & (1 << (i + 4))) == 0 {
            exponent = i;
        } else {
            break;
        }
    }

    let mantissa = if exponent > 0 {
        (sample >> (exponent + 3)) & 0x0F
    } else {
        (sample >> 4) & 0x0F
    };
    
    let mut alaw: i16 = (exponent << 4) | mantissa;
    
    if !sign {
        alaw |= 0x80;
    }

    (alaw as u8) ^ 0x55
}

/// Bu, ITU-T G.711 standardına uygun, endüstri standardı bir µ-Law çevrim algoritmasıdır.
fn pcm_s16_to_ulaw(pcm_val: i16) -> u8 {
    const BIAS: i16 = 0x84;
    const MAX: i16 = 0x7F7F;

    let sign = (pcm_val >> 8) & 0x80;
    let mut sample = if sign != 0 { -pcm_val } else { pcm_val };
    
    if sample > MAX {
        sample = MAX;
    }
    sample += BIAS;

    let mut exponent: i16 = 7;
    for i in (0..=6).rev() {
        if sample <= ((1 << (i + 5)) - 1) {
            exponent = i;
            break;
        }
    }
    
    let mantissa = (sample >> (exponent + 3)) & 0x0F;
    let ulaw: i16 = !((exponent << 4) | mantissa);

    if sign != 0 {
        (ulaw & 0x7F) as u8
    } else {
        (ulaw | 0x80) as u8
    }
}