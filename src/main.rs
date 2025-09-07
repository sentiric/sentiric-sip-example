use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

// Projemizin kütüphane kökünü ve içindeki modülleri kullanıyoruz.
use sentiric_sip_core_service::{
    config::Config,
    error::AppError,
    sip::{self, state::AppState},
    util,
};

fn main() -> Result<(), AppError> {
    let start_time = Instant::now();
    let config = Arc::new(Config::from_env());
    let active_calls: AppState = Arc::new(Mutex::new(Default::default()));

    let socket = Arc::new(UdpSocket::bind(&config.sip_bind_addr)?);
    util::log(start_time, "INFO", "BAŞLATILIYOR", "", &format!("SIP sunucusu {} adresinde dinlemede (Public IP: {}, WAV: {})...", config.sip_bind_addr, config.server_ip, config.wav_file_path));

    let mut buf = [0; 4096];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, remote_addr)) => {
                let data = buf[..len].to_vec();
                
                let sock_clone = Arc::clone(&socket);
                let state_clone = Arc::clone(&active_calls);
                let config_clone = Arc::clone(&config);

                thread::spawn(move || {
                    if let Err(e) = sip::handler::handle_sip_message(start_time, sock_clone, &data, remote_addr, config_clone, state_clone) {
                        util::log(start_time, "ERROR", "İŞLEM HATASI", "", &format!("Gelen istek işlenemedi (Kaynak: {}): {}", remote_addr, e));
                    }
                });
            }
            Err(e) => {
                util::log(start_time, "ERROR", "SOKET HATASI", "", &format!("UDP soketinden veri okunamadı: {}", e));
            }
        }
    }
}