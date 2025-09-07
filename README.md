# Sentric SIP Core Service

[![Build Status](https://github.com/sentiric/sentiric-sip-core-service/actions/workflows/release.yml/badge.svg)](https://github.com/sentiric/sentiric-sip-core-service/actions)

Bu proje, Rust dilinde yazılmış, hafif ve yüksek performanslı bir SIP (Session Initiation Protocol) IVR (Interactive Voice Response) sunucusudur. Temel amacı, harici Rust kütüphanelerine (crate'lere) bağımlı olmadan, gelen SIP çağrılarını karşılamak, önceden belirlenmiş bir ses dosyasını çalmak ve çağrıyı sonlandırmaktır.

Proje, telekom operatörleriyle uyumluluk için G.729 gibi endüstri standardı kodekleri desteklemek amacıyla bir C kütüphanesini Rust FFI (Foreign Function Interface) aracılığıyla entegre eder.

## Özellikler

- **Sıfır Rust Bağımlılığı:** Projenin ana mantığı, Rust'ın standart kütüphanesi dışında hiçbir harici `crate` kullanmaz.
- **Geniş Codec Desteği:**
  - **G.729 (Öncelik 1):** Düşük bant genişliği için `bcg729` C kütüphanesi üzerinden entegre edilmiştir.
  - **G.711 A-law (PCMA)**
  - **G.711 µ-law (PCMU)**
- **Operatör Uyumluluğu:** `Record-Route` başlığını destekleyerek ve uyumsuz kodekler için `488 Not Acceptable Here` yanıtı göndererek telekom standartlarına tam uyum sağlar.
- **Otomatik Dağıtım (CI/CD):** GitHub Actions ile her sürüm için otomatik olarak Docker imajları ve Windows/Linux için hazır çalıştırılabilir dosyalar üretir.

---

## Kullanım Seçenekleri

Bu servisi çalıştırmanın iki yolu vardır. En kolay yöntem Docker kullanmaktır.

### Yöntem 1: Docker ile Çalıştırma (Tavsiye Edilen)

Sisteminizde Docker kuruluysa, herhangi bir derleme işlemi yapmadan servisi tek komutla çalıştırabilirsiniz.

1.  **En son kararlı sürümün imajını çekin:**
    ```bash
    docker pull ghcr.io/sentiric/sentiric-sip-core-service:latest
    ```
    *(Not: Belirli bir sürüm için `:latest` yerine `:v1.0.0` gibi bir etiket kullanın.)*

2.  **Konteyneri çalıştırın:**
    `-e SERVER_IP` değişkenine sunucunuzun dış dünyaya açık (public) IP adresini yazdığınızdan emin olun.

    ```bash
    docker run -d --name sentiric-sip -p 5060:5060/udp \
      -e SERVER_IP="YOUR_PUBLIC_IP_ADDRESS" \
      ghcr.io/sentiric/sentiric-sip-core-service:latest
    ```

3.  **(İsteğe Bağlı) Kendi ses dosyanızı kullanmak için:**
    Kendi `.wav` dosyanızı konteynerin içine bağlayabilir ve `WAV_FILE` ortam değişkeni ile belirtebilirsiniz.
    ```bash
    docker run -d --name sentiric-sip -p 5060:5060/udp \
      -v /yol/sesdosyam.wav:/app/custom.wav \
      -e SERVER_IP="YOUR_PUBLIC_IP_ADDRESS" \
      -e WAV_FILE="/app/custom.wav" \
      ghcr.io/sentiric/sentiric-sip-core-service:latest
    ```

### Yöntem 2: Hazır Derlenmiş Dosyaları Kullanma

Docker kullanmak istemiyorsanız, her sürüm için önceden derlenmiş çalıştırılabilir dosyaları kullanabilirsiniz.

1.  Projenin [**Releases**](https://github.com/sentiric/sentiric-sip-core-service/releases) sayfasına gidin.
2.  En son sürümü bulun ve işletim sisteminize uygun arşivi indirin (`...-windows-x86_64.zip` veya `...-linux-x86_64.tar.gz`).
3.  Arşivi bir klasöre çıkarın. İçinde `sentiric-sip-core-service` (veya `.exe`) ve `welcome.wav` dosyalarını göreceksiniz.
4.  Terminal veya komut istemi üzerinden, dosyaları çıkardığınız klasörde servisi başlatın.

    **Windows (PowerShell):**
    ```powershell
    $env:SERVER_IP = "YOUR_PUBLIC_IP_ADDRESS"
    ./sentiric-sip-core-service.exe
    ```

    **Linux:**
    ```bash
    export SERVER_IP="YOUR_PUBLIC_IP_ADDRESS"
    ./sentiric-sip-core-service
    ```

---

## Geliştiriciler İçin: Kaynaktan Derleme

Eğer kod üzerinde değişiklik yapmak veya projeyi kendiniz derlemek isterseniz, aşağıdaki adımları izleyin.

### Gereksinimler

1.  **Rust Toolchain:** [rustup](https://rustup.rs/)
2.  **Git**
3.  **C Derleme Altyapısı:**
    - **Windows:** Visual Studio Build Tools ("C++ ile Masaüstü geliştirme" iş yükü ile).
    - **Linux (Debian/Ubuntu):** `sudo apt-get install build-essential git`

### Kurulum Adımları

1.  **Ana Projeyi Klonlayın:**
    ```bash
    git clone https://github.com/sentiric/sentiric-sip-core-service.git
    cd sentiric-sip-core-service
    ```

2.  **G.729 C Kütüphanesini Klonlayın:**
    ```bash
    git clone https://github.com/BelledonneCommunications/bcg729.git
    ```

3.  **Projeyi Derleyin ve Çalıştırın:**
    ```bash
    cargo run --release
    ```

## Yapılandırma

Sunucu, aşağıdaki ortam değişkenleri ile yapılandırılır:

-   `SERVER_IP`: Sunucunun genel (public) IP adresi. SDP mesajlarında bu IP adresi kullanılır. (Varsayılan: `127.0.0.1`)
-   `SIP_PORT`: Sunucunun dinleyeceği SIP portu. (Varsayılan: `5060`)
-   `WAV_FILE`: Çalınacak olan ses dosyasının yolu. Dosya formatı **8000Hz, 16-bit, Mono PCM (`.wav`)** olmalıdır. (Varsayılan: `welcome.wav`)