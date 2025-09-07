# Sentric SIP IVR Sunucusu - Entegrasyon Teknik Şartnamesi

**Belge Sürümü:** 1.1 (Kararlı Sürüm)
**Tarih:** 2025-09-07
**Amaç:** Bu belge, Sentric SIP IVR sunucusunun telekom operatörü altyapısıyla entegrasyon için sunduğu teknik yetenekleri ve uyumluluk standartlarını tanımlar.

---

### 1. Sinyalleşme Protokolü (SIP/SDP)

#### 1.1. SIP Başlık (Header) Yönetimi

| Başlık Adı      | Uyumluluk Durumu | Açıklama                                                                                                                                                                                              |
| :-------------- | :--------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`Record-Route`** | **Tam Destek**   | Sunucu, gelen `INVITE` mesajındaki `Record-Route` başlığını saklar ve çağrı yaşam döngüsü boyunca gönderdiği sonraki isteklere (örn: `BYE`) bu başlığı ekler. Bu, operatör proxy'leri arkasında doğru çalışmayı garanti eder. |
| **`Via`**         | **Standart Uyum**  | Sunucu, gelen istekteki en üst `Via` başlığını, gönderdiği yanıtlara (`100 Trying`, `200 OK` vb.) standartlara uygun şekilde kopyalar.                                                                 |

#### 1.2. Oturum Açıklama Protokolü (SDP) Yönetimi

Sunucu, gelen `INVITE` içindeki SDP teklifini aşağıdaki yeteneklere göre işler.

##### 1.2.1. Ses Codec Anlaşması

Sunucu, gelen SDP'deki codec listesini aşağıdaki öncelik sırasıyla değerlendirir ve desteklediği ilk kodeği seçer.

| Öncelik | Payload Tipi | Codec Adı            | Sunucu Desteği  | Açıklama                                                            |
| :------ | :----------- | :------------------- | :-------------- | :------------------------------------------------------------------ |
| **1**   | **18**       | **G.729**            | **Destekleniyor** | Düşük bant genişliği (8 kbps) için optimize edilmiş standart kodek.  |
| **2**   | **8**        | **G.711 A-law (PCMA)** | **Destekleniyor** | Avrupa ve dünya genelinde yaygın telekom standardı.                 |
| **3**   | **0**        | **G.711 u-Law (PCMU)** | **Destekleniyor** | Kuzey Amerika ve Japonya'da yaygın telekom standardı.                 |

##### 1.2.2. Medya Paketleme ve Zamanlama

| SDP Alanı   | Beklenen Değer | Sunucu Davranışı                                                                                                                   |
| :---------- | :------------- | :--------------------------------------------------------------------------------------------------------------------------------- |
| **`a=ptime`** | `20`           | Sunucu, tüm desteklenen kodekler için 20 milisaniye (ms) ses içeren RTP paketleri gönderir. Bu, operatör beklentileriyle tam uyumludur. |
| **`a=rtpmap`**  | `101`          | Sunucu, DTMF (tuşlama) sinyallerini **işlemez** ancak varlıkları çağrının kurulmasına engel olmaz. Gelecekteki geliştirmeler için uyumludur. |

---

### 2. Medya Akışı (RTP)

| Özellik        | Yetenek                                                                                                 |
| :------------- | :------------------------------------------------------------------------------------------------------ |
| **Kaynak Medya** | **8000Hz, 16-bit, Mono PCM** formatındaki `.wav` dosyalarını okuyabilir.                                   |
| **Kodlama**      | Okunan PCM verisini, SDP'de anlaşılan kodeğe (G.729, PCMA, veya PCMU) canlı olarak kodlar.                |
| **Zamanlama**    | RTP paketlerini 20ms aralıklarla, hassas zamanlama ile göndererek `jitter` (gecikme değişkenliği) en aza indirir. |