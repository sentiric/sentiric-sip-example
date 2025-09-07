use crate::error::AppError;
use std::fs::File;
use std::io::Read;

pub fn read_wav_file(path: &str) -> Result<Vec<i16>, AppError> {
    let mut file = File::open(path)
        .map_err(|e| AppError::WavFileError(format!("'{}' dosyası açılamadı: {}", path, e)))?;
    
    let mut header = [0u8; 44];
    file.read_exact(&mut header)?;

    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" || &header[20..22] != [1, 0] 
        || &header[22..24] != [1, 0] || &header[24..28] != [0x40, 0x1f, 0, 0] // 8000 Hz
        || &header[34..36] != [16, 0] // 16-bit
    {
        return Err(AppError::WavFileError(format!("'{}' dosyası formatı desteklenmiyor. Lütfen 8000Hz, 16-bit, Mono PCM formatında kullanın.", path)));
    }
    
    let mut pcm_bytes = Vec::new();
    file.read_to_end(&mut pcm_bytes)?;
    Ok(pcm_bytes.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect())
}