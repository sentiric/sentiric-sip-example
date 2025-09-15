use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Standart formatta loglama yapan fonksiyon.
pub fn log(start_time: Instant, level: &str, log_type: &str, event: &str, message: &str) {
    let elapsed = start_time.elapsed();
    println!("[{:09.3}] [{}] [{:<16}] [sentiric] [{:<5}] {}", elapsed.as_secs_f32(), level, log_type, event, message);
}

/// Mevcut zamanı milisaniye cinsinden Unix Epoch'tan bu yana döner.
pub fn get_timestamp_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}