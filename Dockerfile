# --- Aşama 1: Derleme Ortamı (Builder) ---
# Rust'ın resmi, tam donanımlı imajını temel alıyoruz.
# Bu imaj, Rust derleyicisi, C derleyicisi (Clang), Git ve diğer
# gerekli araçları içerir.
FROM rust:1.79 as builder

# Derleme için çalışma dizini oluştur
WORKDIR /usr/src/app

# Önce sadece bağımlılık dosyalarını kopyala. Bu sayede, sadece kod
# değiştiğinde bağımlılıkları tekrar indirmek zorunda kalmayız (Docker cache).
COPY Cargo.toml Cargo.lock ./
# Boş bir lib.rs oluşturarak sadece bağımlılıkları derle
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release

# Şimdi gerçek kodumuzu kopyala
COPY src ./src
COPY build.rs ./build.rs

# bcg729 C kütüphanesini klonla
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*
RUN git clone https://github.com/BelledonneCommunications/bcg729.git

# Önbelleğe alınmış bağımlılıkları kullanarak projeyi derle
RUN cargo build --release

# --- Aşama 2: Çalıştırma Ortamı (Runner) ---
# Debian'ın son derece küçük ve hafif "slim" versiyonunu temel alıyoruz.
# Bu, son imaj boyutunu önemli ölçüde küçültür.
FROM debian:bookworm-slim

# --- Çalışma zamanı sistem bağımlılıkları ---
RUN apt-get update && apt-get install -y --no-install-recommends \
    netcat-openbsd \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* 

# Derlenmiş olan binary dosyasını Builder aşamasından bu aşamaya kopyala
COPY --from=builder /usr/src/app/target/release/sentiric-sip-core-service /usr/local/bin/sentiric-sip-core-service

# Uygulamanın çalışması için gereken .wav dosyasını imajın içine kopyala
# Bu sayede kullanıcıların ayrıca bir dosya indirmesine gerek kalmaz.
COPY welcome.wav /app/welcome.wav

# Uygulamanın çalışacağı dizini ayarla
WORKDIR /app

# Konteyner başlatıldığında çalıştırılacak olan komut
# Varsayılan olarak welcome.wav dosyasını kullanacak şekilde ayarlandı.
ENV WAV_FILE=/app/welcome.wav
CMD ["sentiric-sip-core-service"]