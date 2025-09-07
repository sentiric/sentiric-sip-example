# Sentric SIP Core Service

Bu proje, Rust dilinde yazılmış, hafif ve yüksek performanslı bir SIP (Session Initiation Protocol) IVR (Interactive Voice Response) sunucusudur. Temel amacı, harici Rust kütüphanelerine (crate'lere) bağımlı olmadan, gelen SIP çağrılarını karşılamak, önceden belirlenmiş bir ses dosyasını çalmak ve çağrıyı sonlandırmaktır.

Proje, telekom operatörleriyle uyumluluk için G.729 gibi endüstri standardı kodekleri desteklemek amacıyla bir C kütüphanesini Rust FFI (Foreign Function Interface) aracılığıyla entegre eder.

## Özellikler

- **Sıfır Rust Bağımlılığı:** Projenin ana mantığı, Rust'ın standart kütüphanesi dışında hiçbir harici `crate` kullanmaz.
- **SIP Temel Akış Desteği:** `INVITE`, `ACK` ve `BYE` metodlarını işleyerek tam bir çağrı yaşam döngüsünü yönetir.
- **Geniş Codec Desteği:**
  - **G.729 (Öncelik 1):** Düşük bant genişliği için C kütüphanesi (`bcg729`) üzerinden entegre edilmiştir.
  - **G.711 A-law (PCMA)**
  - **G.711 µ-law (PCMU)**
- **Operatör Uyumluluğu:** `Record-Route` başlığını destekleyerek telekom operatörü proxy'leri arkasında doğru çalışır.
- **Medya Akışı:** Belirtilen bir `.wav` dosyasını (8000Hz, 16-bit, Mono PCM) anlaşılan kodek ile kodlayarak RTP üzerinden canlı olarak akıtır.
- **Yapılandırılabilirlik:** Ortam değişkenleri (environment variables) aracılığıyla kolayca yapılandırılabilir.

## Gereksinimler

Bu projeyi derleyip çalıştırabilmek için sisteminizde aşağıdakilerin kurulu olması gerekir:

1.  **Rust Toolchain:** [rustup](https://rustup.rs/) aracılığıyla yüklenmiş güncel bir Rust derleyicisi.
2.  **Git:** `bcg729` C kütüphanesini klonlamak için gereklidir.
3.  **C Derleme Altyapısı (Toolchain):**
    - **Windows:** [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) gereklidir. Kurulum sırasında **"C++ ile Masaüstü geliştirme" (Desktop development with C++)** iş yükünü seçtiğinizden emin olun.
    - **Linux (Debian/Ubuntu):** `sudo apt-get install build-essential`
    - **macOS:** Xcode Command Line Tools

## Kurulum ve Çalıştırma

1.  **Ana Projeyi Klonlayın:**
    ```bash
    git clone <bu_projenin_git_adresi>
    cd sentiric-sip-core-service
    ```

2.  **G.729 C Kütüphanesini Klonlayın:**
    Projenin `build.rs` script'i, C kodunu derlemek için kaynak dosyaları projenin içinde arar. `bcg729` kütüphanesini projenin kök dizinine klonlayın.
    ```bash
    git clone https://github.com/BelledonneCommunications/bcg729.git
    ```
    Bu komut sonrasında proje dizininizde `bcg729` adında bir klasör oluşmalıdır.

3.  **Projeyi Derleyin ve Çalıştırın:**
    Artık projeyi derlemeye hazırsınız. `--release` bayrağı, üretim ortamları için optimize edilmiş bir çıktı oluşturur.
    ```bash
    cargo run --release
    ```
    Derleme işlemi, önce `bcg729` C kütüphanesini, ardından Rust kodunu derleyecektir. İşlem tamamlandığında sunucu otomatik olarak başlayacaktır.

## Yapılandırma

Sunucu, aşağıdaki ortam değişkenleri ile yapılandırılır:

-   `SERVER_IP`: Sunucunun genel (public) IP adresi. SDP mesajlarında bu IP adresi kullanılır. (Varsayılan: `127.0.0.1`)
-   `SIP_PORT`: Sunucunun dinleyeceği SIP portu. (Varsayılan: `5060`)
-   `WAV_FILE`: Çalınacak olan ses dosyasının yolu. Dosya formatı **8000Hz, 16-bit, Mono PCM (`.wav`)** olmalıdır. (Varsayılan: `welcome.wav`)

**Örnek (Linux/macOS):**
```bash
export SERVER_IP="192.0.2.10"
export WAV_FILE="/opt/sounds/greeting.wav"
cargo run --release```

**Örnek (Windows PowerShell):**
```powershell
$env:SERVER_IP = "192.0.2.10"
$env:WAV_FILE = "C:\sounds\greeting.wav"
cargo run --release
```