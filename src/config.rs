use std::env;

/// Uygulama yapılandırmasını tutan yapı.
pub struct Config {
    pub server_ip: String,
    pub sip_port: String,
    pub sip_bind_addr: String,
    pub wav_file_path: String,
}

impl Config {
    /// Ortam değişkenlerinden yapılandırmayı okur veya varsayılan değerleri kullanır.
    pub fn from_env() -> Self {
        let server_ip = env::var("SERVER_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
        let sip_port = env::var("SIP_PORT").unwrap_or_else(|_| "5060".to_string());
        let sip_bind_addr = format!("0.0.0.0:{}", sip_port);
        let wav_file_path = env::var("WAV_FILE").unwrap_or_else(|_| "welcome.wav".to_string());

        Config {
            server_ip,
            sip_port,
            sip_bind_addr,
            wav_file_path,
        }
    }
}