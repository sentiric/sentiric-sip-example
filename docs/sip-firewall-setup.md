# SIP Firewall ve Trafik İzleme Rehberi

Bu döküman, SIP protokolü kullanan VoIP trafiğini analiz etmek, izlemek ve firewall üzerinde doğru yapılandırmaları yapmak için kapsamlı bir rehberdir. Operatör IP'si olarak örnekte kullanılan adres: `194.48.95.2`

---

## 📦 Gereken Araçlar

### Linux Sistemler için Kurulum:
```bash
sudo apt-get update && sudo apt-get install tcpdump ngrep -y
````

### Windows Sistemler için:

```cmd
netstat -a -n -o | find "5060"
taskkill /F /PID <PID>
```

---

## 🔍 SIP Trafiği İzleme

### `tcpdump` ile:

```bash
sudo tcpdump -i any -n udp port 5060 -A
sudo tcpdump -i any portrange 5060-5070 -n -A
sudo tcpdump -i any portrange 10000-20000 -n -A  # RTP ses trafiği
```

### `ngrep` ile:

```bash
sudo ngrep -d any -W byline port 5060
```

---

## 📄 Örnek SIP Trafiği

Aşağıda, `194.48.95.2` IP adresinden gelen bir SIP INVITE mesajı ve devamındaki sinyalleşme örneği yer alır:

```log
interface: any
filter: ( port 5060 ) and (ip || ip6)
#
U 194.48.95.2:5060 -> 10.128.0.17:5060 #1
INVITE sip:902124548590@34.122.40.122:5060 SIP/2.0.
Record-Route: <sip:194.48.95.2;trasport=udp;ftag=f5598e370425cf189726907e91a251d4;lr>.
Via: SIP/2.0/UDP 194.48.95.2:5060;branch=z9hG4bK71ca.f0c036f6bd1fc800b7c2a945eff1bcb5.0.
Via: SIP/2.0/UDP 194.48.95.2:5065;received=194.48.95.2;rport=5065;branch=z9hG4bK68ba4cb51e91e565f2cb92362a948830.
Max-Forwards: 69.
From: <sip:05548777858@194.48.95.2>;tag=f5598e370425cf189726907e91a251d4.
To: <sip:902124548590@34.122.40.122>.
Call-ID: 021ACF557981400001A18556@TS_ROITEL_TRUST-b2b_1-onnet_1-b2b_1.
CSeq: 200 INVITE.
Contact: <sip:05548777858@194.48.95.2:5065>.
Expires: 300.
User-Agent: Sippy Softswitch v2021-PRODUCTION.408.
Content-Length: 174.
Content-Type: application/sdp.
.
v=0.
o=- 53646969 1 IN IP4 192.0.2.1.
s=-.
c=IN IP4 194.48.95.2.
t=0 0.
m=audio 20728 RTP/AVP 18 8 0 101.
a=fmtp:18 annexb=no.
a=rtpmap:101 telephone-event/8000.
a=ptime:20.

#
U 10.128.0.17:5060 -> 194.48.95.2:5060 #2
SIP/2.0 100 Trying.
Via: SIP/2.0/UDP 194.48.95.2:5060;branch=z9hG4bK71ca.f0c036f6bd1fc800b7c2a945eff1bcb5.0.
Via: SIP/2.0/UDP 194.48.95.2:5065;received=194.48.95.2;rport=5065;branch=z9hG4bK68ba4cb51e91e565f2cb92362a948830.
From: <sip:05548777858@194.48.95.2>;tag=f5598e370425cf189726907e91a251d4.
To: <sip:902124548590@34.122.40.122>.
Call-ID: 021ACF557981400001A18556@TS_ROITEL_TRUST-b2b_1-onnet_1-b2b_1.
CSeq: 200 INVITE.
Content-Length: 0.
.

#
U 10.128.0.17:5060 -> 194.48.95.2:5060 #3
SIP/2.0 200 OK.
Via: SIP/2.0/UDP 194.48.95.2:5060;branch=z9hG4bK71ca.f0c036f6bd1fc800b7c2a945eff1bcb5.0.
Via: SIP/2.0/UDP 194.48.95.2:5065;received=194.48.95.2;rport=5065;branch=z9hG4bK68ba4cb51e91e565f2cb92362a948830.
From: <sip:05548777858@194.48.95.2>;tag=f5598e370425cf189726907e91a251d4.
To: <sip:902124548590@34.122.40.122>;tag=19928441604.
Call-ID: 021ACF557981400001A18556@TS_ROITEL_TRUST-b2b_1-onnet_1-b2b_1.
CSeq: 200 INVITE.
Contact: <sip:34.122.40.122:5060>.
Content-Type: application/sdp.
Content-Length: 139.
.
v=0.
o=- 0 0 IN IP4 34.122.40.122.
s=sentiric.
c=IN IP4 34.122.40.122.
t=0 0.
m=audio 19576 RTP/AVP 18.
a=rtpmap:18 G729/8000.
a=sendrecv.

#
U 194.48.95.2:5065 -> 10.128.0.17:5060 #4
ACK sip:34.122.40.122:5060 SIP/2.0.
Via: SIP/2.0/UDP 194.48.95.2:5065;rport;branch=z9hG4bK50373062f5d41433679feebf2bb670bd.
Max-Forwards: 70.
From: <sip:05548777858@194.48.95.2>;tag=f5598e370425cf189726907e91a251d4.
To: <sip:902124548590@34.122.40.122>;tag=19928441604.
Call-ID: 021ACF557981400001A18556@TS_ROITEL_TRUST-b2b_1-onnet_1-b2b_1.
CSeq: 200 ACK.
User-Agent: Sippy Softswitch v2021-PRODUCTION.408.
X-socket: udp:194.48.95.2.
Content-Length: 0.
.

#
U 10.128.0.17:5060 -> 194.48.95.2:5060 #5
BYE sip:05548777858@194.48.95.2:5065 SIP/2.0.
Via: SIP/2.0/UDP 34.122.40.122:5060;branch=z9hG4bK.199284439f5.
Record-Route: <sip:194.48.95.2;trasport=udp;ftag=f5598e370425cf189726907e91a251d4;lr>.
From: <sip:sentiric@34.122.40.122>;tag=19928441604.
To: <sip:05548777858@194.48.95.2>;tag=f5598e370425cf189726907e91a251d4.
Call-ID: 021ACF557981400001A18556@TS_ROITEL_TRUST-b2b_1-onnet_1-b2b_1.
CSeq: 210 BYE.
Max-Forwards: 70.
Content-Length: 0.
.

#
U 194.48.95.2:5060 -> 10.128.0.17:5060 #6
SIP/2.0 200 OK.
Via: SIP/2.0/UDP 34.122.40.122:5060;branch=z9hG4bK.199284439f5.
Record-Route: <sip:194.48.95.2;trasport=udp;ftag=f5598e370425cf189726907e91a251d4;lr>.
From: <sip:sentiric@34.122.40.122>;tag=19928441604.
To: <sip:05548777858@194.48.95.2>;tag=f5598e370425cf189726907e91a251d4.
Call-ID: 021ACF557981400001A18556@TS_ROITEL_TRUST-b2b_1-onnet_1-b2b_1.
CSeq: 210 BYE.
Server: Sippy Softswitch v2021-PRODUCTION.408.
Content-Length: 0.
.


```

> Bu log'lar SIP bağlantısının `5060`, `5061` gibi UDP portlarını ve RTP için `15502` gibi portları kullandığını gösteriyor.

---

## 🔐 Firewall Kurulumu

### Güvenli IP Tanımı

Sadece `194.48.95.2` adresine izin vermek için `CIDR /32` kullanılır:

#### `iptables`:

```bash
iptables -A INPUT -p udp -s 194.48.95.2/32 --dport 5060 -j ACCEPT
iptables -A INPUT -p udp -s 194.48.95.2/32 --dport 5070 -j ACCEPT
```

#### `ufw` (Ubuntu):

```bash
sudo ufw allow from 194.48.95.2/32 to any port 5060 proto udp
sudo ufw allow from 194.48.95.2/32 to any port 5070 proto udp
```

---

### 🎧 RTP Ses Trafiği

RTP ses akışı için genellikle UDP 10000–20000 port aralığı kullanılır:

```bash
iptables -A INPUT -p udp --dport 10000:20000 -j ACCEPT
```

> Port aralığını operatörünüzden net olarak öğrenin. Örneğin, `15502` ve `15182` gibi portlar log'larda görünüyor.

---

## 🧠 CIDR Notasyonu

| CIDR  | Anlamı                       | Açıklama                      |
| ----- | ---------------------------- | ----------------------------- |
| `/32` | Tek bir IP                   | En güvenli tercih             |
| `/24` | 256 IP (örneğin x.x.x.0-255) | Geniş ağ, sadece gerektiğinde |

> Tavsiye: Sadece ihtiyaç duyduğun IP’ye izin ver (`/32`), tüm bloğa değil.

---

## 🛠 Sık Karşılaşılan Hatalar

### 🔒 SIP trafiği gelmiyor

* IP adresi doğru mu?
* UDP 5060 ve 5070 portları açık mı?
* NAT ya da port yönlendirme yapılmış mı?

### 🔇 Ses gelmiyor

* RTP portları (`10000–20000`) açık mı?
* Codec uyumsuzluğu olabilir

---

## 📁 Repo Yapısı (Önerilen)

```
📂 sip-tools-docs
├── README.md
├── sip-firewall-setup.md
├── sip-troubleshooting.md
└── examples/
    ├── sample-ngrep.log
    └── sample-tcpdump.log
```

---

## 📌 Kaynak IP Örneği

Aşağıdaki IP, operatör tarafında kullanılan IP’dir:

```text
194.48.95.2
```

Firewall’a tanımlanırken mutlaka `/32` CIDR ile girilmelidir.

---

## 🧩 Ekstra: SIP NAT ve ALG Uyarısı

Bazı modem/router cihazları SIP ALG (Application Layer Gateway) özelliği içerir. Bu özellik genelde SIP paketlerini değiştirerek sorun yaratabilir.

> **Tavsiye:** Modem/Router cihazınızda SIP ALG özelliğini kapatın.

---

## Lisans

Bu dokümantasyon MIT lisansı ile paylaşılmıştır. Dilediğiniz gibi kullanabilir, geliştirebilir ve dağıtabilirsiniz.

---

