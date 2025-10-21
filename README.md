# 🚀 **Projek-SKT2025-Kelompok7**

> 💡 *Proyek Sistem Kontrol Terdistribusi menggunakan ESP32 S3 dan Rust*

---

## 🌐 **Deskripsi Proyek**
Proyek ini merupakan tugas dari mata kuliah **Sistem Kontrol Terdistribusi (SKT)**.  
Kelompok 7 merancang sebuah sistem kontrol yang memiliki fitur serupa dengan **Distributed Control System (DCS)**, dengan menggunakan **ESP32 S3** sebagai pusat kendali utama.

---

## ⚙️ **Komponen yang Digunakan**

| Komponen | Fungsi |
|-----------|---------|
| **ESP32 S3** | Mikrokontroler utama sebagai pusat kendali |
| **Sensor SHT20** | Mengukur suhu dan kelembapan |
| **MAX485 RS485 TTL** | Komunikasi serial jarak jauh antar perangkat |
| **Relay** | Aktuator untuk mengendalikan beban listrik |
| **Buzzer** | Indikator suara / alarm sistem |

---

## 🧠 **Bahasa Pemrograman**
Proyek dikembangkan menggunakan **Rust**, dengan dukungan ekosistem **ESP-IDF**.

### Instalasi awal:
```bash
cargo generate --git https://github.com/esp-rs/esp-idf-template.git
Pilih ESP32 S3 dan gunakan versi ESP-IDF 5.3 untuk kestabilan.

🔧 Langkah Instalasi & Konfigurasi
Instal toolchain pendukung:

bash
Copy code
cargo install espup
cargo install espflash
Cek koneksi board:

bash
Copy code
espflash board-info
Tambahkan dependensi di Cargo.toml

WiFi dan TCP/IP

Modul InfluxDB

Modul UART RS485

Library sensor & aktuator

💻 Struktur Program (main.rs)
Program utama mengatur:

Inisialisasi koneksi WiFi

Pengiriman data ke InfluxDB

Integrasi dengan ThingsBoard

Pembacaan sensor via RS485

Kontrol Relay dan Buzzer

📡 Koneksi ke InfluxDB Lokal
Agar ESP32 dapat mengirim data ke InfluxDB, pastikan:

Memiliki ORG ID, Bucket, dan Token

IP laptop (server) dan ESP32 berada pada subnet yang sama

🏠 Analogi subnet:
Laptop dan ESP32 harus berada dalam “komplek perumahan WiFi” yang sama.
Namun, alamat rumahnya (IP) harus berbeda agar tidak tabrakan.

☁️ Integrasi ThingsBoard
Buka demo.thingsboard.io

Aktifkan device dan salin access token

Masukkan token ke dalam main.rs

Jalankan program untuk menampilkan hasil sensor di dashboard ThingsBoard

🔌 Fitur Utama
✅ Pembacaan data suhu & kelembapan (SHT20)
✅ Komunikasi RS485 antar perangkat
✅ Pengiriman data ke InfluxDB lokal
✅ Monitoring online melalui ThingsBoard
✅ Kontrol relay dan buzzer

👥 Tim Kelompok 7
- Adrian Yared Immanuel (2042221080)
- Muhammad Yusron Maskur (2042231030
- Agus Wedi (2042231066)
Mata Kuliah Sistem Kontrol Terdistribusi (SKT)
📍 Tahun Akademik 2025
