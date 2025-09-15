# Sentric SIP IVR Sunucusu - Entegrasyon Teknik Şartnamesi

**Belge Sürümü:** 1.2 (Operatör Uyumlu Kararlı Sürüm)
**Tarih:** 2025-09-08
**Amaç:** Bu belge, Sentric SIP IVR sunucusunun telekom operatörü altyapısıyla entegrasyon için sunduğu teknik yetenekleri ve uyumluluk standartlarını tanımlar.

---

### 1. Sinyalleşme Protokolü (SIP/SDP)

Sunucu, RFC 3261 standardına uygun olarak çalışır ve özellikle operatör proxy (SBC) sistemleriyle uyumluluk için kritik olan başlık (header) yönetimi kurallarına sıkı sıkıya uyar.

#### 1.1. SIP Başlık (Header) Yönetimi

| Başlık Adı      | Uyumluluk Durumu | Açıklama                                                                                                                                                                                              |
| :-------------- | :--------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`Via`**         | **Tam Uyumlu**     | Sunucu, gelen bir istekteki **tüm `Via` başlıklarını** sırasını bozmadan alır ve gönderdiği tüm yanıtlara (`100 Trying`, `200 OK` vb.) ekler. Bu, operatör proxy'leri üzerinden geçen çağrıların doğru yönlendirilmesi için zorunludur ve tam olarak desteklenmektedir. |
| **`Record-Route`** | **Tam Uyumlu**   | Sunucu, gelen `INVITE` mesajındaki `Record-Route` başlığını saklar ve çağrı yaşam döngüsü boyunca gönderdiği sonraki isteklere (örn: `BYE`) bu başlığı ekler. Bu, çağrının sinyalleşme yolunun korunmasını garanti eder. |

#### 1.2. Oturum Açıklama Protokolü (SDP) Yönetimi

Sunucu, gelen `INVITE` içindeki SDP teklifini aşağıdaki yeteneklere göre işler.

##### 1.2.1. Ses Codec Anlaşması

Sunucu, gelen SDP'deki codec listesini ("m=audio" satırı) aşağıdaki öncelik sırasıyla değerlendirir ve desteklediği ilk kodeği seçerek `200 OK` yanıtında bildirir.

| Öncelik | Payload Tipi | Codec Adı            | Sunucu Desteği  | Açıklama                                                            |
| :------ | :----------- | :------------------- | :-------------- | :------------------------------------------------------------------ |
| **1**   | **18**       | **G.729**            | **Destekleniyor** | Düşük bant genişliği (8 kbps) için optimize edilmiş, operatörlerin tercih ettiği standart kodek.  |
| **2**   | **8**        | **G.711 A-law (PCMA)** | **Destekleniyor** | Avrupa ve dünya genelinde yaygın telekom standardı.                 |
| **3**   | **0**        | **G.711 µ-Law (PCMU)** | **Destekleniyor** | Kuzey Amerika ve Japonya'da yaygın telekom standardı.                 |

##### 1.2.2. Uyumsuz Codec Durumu

Eğer gelen SDP teklifinde yukarıdaki üç kodekten hiçbiri bulunmazsa, sunucu çağrıyı kurmaz ve standartlara uygun olarak **`488 Not Acceptable Here`** hatası ile `INVITE` isteğini reddeder.

##### 1.2.3. Medya Paketleme ve Zamanlama

| SDP Alanı   | Beklenen/Sunulan Değer | Sunucu Davranışı                                                                                                                   |
| :---------- | :--------------------- | :--------------------------------------------------------------------------------------------------------------------------------- |
| **`a=ptime`** | `20`                   | Sunucu, tüm desteklenen kodekler için 20 milisaniye (ms) ses içeren RTP paketleri gönderir. Bu, operatör beklentileriyle tam uyumludur. |
| **`a=sendrecv`**| `sendrecv`             | Sunucu, medya akışının iki yönlü olabileceğini belirtir. Mevcut implementasyonda sadece ses gönderse de (`sendonly`), gelecekteki geliştirmeler için uyumluluk sağlar. |
| **`a=rtpmap`**  | `101` (DTMF)           | Sunucu, DTMF (tuşlama) sinyallerini **işlemez** ancak varlıkları çağrının kurulmasına engel olmaz. Gelecekteki geliştirmeler için uyumludur. |

---

### 2. Medya Akışı (RTP)

| Özellik        | Yetenek                                                                                                 |
| :------------- | :------------------------------------------------------------------------------------------------------ |
| **Kaynak Medya** | **8000Hz, 16-bit, Mono PCM** formatındaki `.wav` dosyalarını okuyabilir.                                   |
| **Kodlama**      | Okunan PCM verisini, SDP'de anlaşılan kodeğe (G.729, PCMA, veya PCMU) canlı olarak kodlar.                |
| **Zamanlama**    | RTP paketlerini 20ms aralıklarla, hassas zamanlama ile göndererek `jitter` (gecikme değişkenliği) en aza indirir. |