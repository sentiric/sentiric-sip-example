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

## Yapılandırma (Ortam Değişkenleri)

Uygulamayı çalıştırmadan önce aşağıdaki ortam değişkenlerini ayarlamanız gerekmektedir:

*   **`SERVER_IP` (Zorunlu):** Sunucunun dış dünyaya tanıtacağı IP adresi. Yerel testler için `192.168.1.3` gibi yerel IP'nizi kullanabilirsiniz.
*   **`SIP_PORT` (İsteğe bağlı):** SIP isteklerini dinleyeceği port. Ayarlanmazsa varsayılan olarak `5060` kullanılır.
*   **`WAV_FILE` (İsteğe bağlı):** Çalınacak olan `.wav` dosyasının yolu. Ayarlanmazsa varsayılan olarak `welcome.wav` kullanılır.

### Çalıştırma Örneği (Windows PowerShell)

```powershell
# Gerekli değişkenleri ayarla
$env:SERVER_IP="192.168.1.3"

# (İsteğe bağlı) Diğer değişkenleri ayarla
$env:SIP_PORT="5080"
$env:WAV_FILE="sounds/greeting.wav"

# Programı çalıştır
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

