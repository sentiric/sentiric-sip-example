# Sentiric - Karşılama Servisi (Rust Versiyonu - Sıfır Bağımlılık)

Bu proje, gelen telefon aramalarını karşılayan, önceden belirlenmiş bir ses dosyasını çalan ve ardından aramayı sonlandıran basit bir VoIP servisinin (B2BUA), **hiçbir harici bağımlılık olmadan** tamamen Rust standart kütüphanesi ile yazılmış prototipidir.

## Projenin Amacı

Dışarıdan gelen SIP `INVITE` isteklerine cevap vererek, sunucuda bulunan `welcome.wav` dosyasını arayana RTP protokolü üzerinden dinletmek ve dosya bittiğinde aramayı otomatik olarak `BYE` isteği ile sonlandırmak.

## Teknik Gereksinimler

*   **Dil:** Rust
*   **Bağımlılık:** **Sıfır harici bağımlılık.** Proje sadece Rust standart kütüphanesini kullanır. `cargo build` komutu internet bağlantısı gerektirmez.
*   **Harici Bağımlılıklar:** Asterisk, FreeSWITCH gibi harici PBX yazılımlarına ihtiyaç **duymaz**.

## Kurulum

1.  **Rust Kurulumu:**
    Sisteminizde Rust'ın kurulu olduğundan emin olun.
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2.  **Proje Dosyaları:**
    Bu dizindeki `Cargo.toml` ve `src/main.rs` dosyalarını oluşturun.

3.  **Derleme:**
    Proje ana dizinindeyken aşağıdaki komutu çalıştırın. Cargo, hiçbir şey indirmeden projeyi doğrudan derleyecektir.
    ```bash
    cargo build --release
    ```
    `--release` bayrağı, performans için optimizasyonları aktif hale getirir.

## Yapılandırma

1.  **Sunucu IP Adresi:**
    Uygulamanın çalışacağı sunucunun genel (public) IP adresini `src/main.rs` dosyasının en üstündeki `SERVER_IP` sabitinde belirtmeniz gerekmektedir.
    ```rust
    // src/main.rs
    const SERVER_IP: &str = "34.122.40.122"; // <-- BU SATIRI KENDİ SUNUCU IP'NİZ İLE DEĞİŞTİRİN
    ```

2.  **Ses Dosyası:**
    Çalınacak ses dosyasının `welcome.wav` adıyla ana proje dizininde bulunduğundan emin olun. Dosyanın formatı **PCM, 16-bit, 8000 Hz, Mono** olmalıdır. Uygulama, bu formatı G.711 A-Law formatına otomatik olarak çevirecektir.

## Çalıştırma

Derleme sonrası oluşan çalıştırılabilir dosyayı doğrudan çalıştırabilirsiniz:

```bash
# Windows için:
target\release\sentiric-sip-core-service.exe

# Linux / macOS için:
./target/release/sentiric-sip-core-service
```

Veya `cargo` ile de çalıştırabilirsiniz:
```bash
cargo run --release
```

Uygulama başarıyla başladığında konsolda aşağıdaki gibi bir çıktı göreceksiniz:

```
[INFO] [BAŞLATILIYOR] [sentiric-sip-core-service] SIP sunucusu 34.122.40.122:5060 adresinde dinlemede...
```

## Loglama

Tüm arama aktiviteleri standart çıktıya (konsola) yazdırılacaktır. (Not: Zaman damgası formatı, harici kütüphane kullanılmadığı için daha basittir.)

```
[INFO] [ARAMA BAŞLADI] [sentiric-sip-core-service] [ÇAĞRI] Arayan: +905548777858, Aranan: +902124548590 Kaynak: 194.48.95.2:5060 HEDEF: 34.122.40.122:5060
[INFO] [ARAMA CEVAPLANDI] [sentiric-sip-core-service] [CEVAP] Arayan: +905548777858, Aranan: +902124548590 Kaynak: 194.48.95.2:5060 HEDEF: 34.122.40.122:5060
[INFO] [RTP BAŞLADI] [sentiric-sip-core-service] [SES] Arayan: +905548777858, Aranan: +902124548590 Kaynak: 194.48.95.2:5060 HEDEF: 34.122.40.122:5060
[INFO] [WAV BİTTİ] [sentiric-sip-core-service] [WAV] Arayan: +905548777858, Aranan: +902124548590 Kaynak: 194.48.95.2:5060 HEDEF: 34.122.40.122:5060
[INFO] [ARAMA SONLANDI] [sentiric-sip-core-service] [KAPAT] Arayan: +905548777858, Aranan: +902124548590 Kaynak: 194.48.95.2:5060 HEDEF: 34.122.40.122:5060
```

