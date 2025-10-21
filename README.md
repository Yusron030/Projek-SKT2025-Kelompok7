# Projek-SKT2025-Kelompok7
🌐 Sistem Kontrol Terdistribusi Berbasis ESP32 S3 dan Rust

Proyek ini merupakan tugas dari mata kuliah Sistem Kontrol Terdistribusi (SKT).
Kelompok 7 merancang sebuah sistem kontrol yang memiliki fitur serupa dengan Distributed Control System (DCS), dengan menggunakan ESP32 S3 sebagai pusat kendali utama.

⚙️ Komponen yang Digunakan

ESP32 S3

Sensor SHT20 – untuk pengukuran suhu dan kelembapan

MAX485 RS485 TTL – komunikasi serial jarak jauh antar perangkat

Relay – sebagai aktuator untuk mengendalikan beban

Buzzer – sebagai indikator suara

🧠 Bahasa Pemrograman

Proyek ini dikembangkan menggunakan Rust, dengan bantuan ekosistem ESP-IDF.

Langkah awal instalasi:

cargo generate --git https://github.com/esp-rs/esp-idf-template.git


Kemudian pilih chip ESP32-S3 dengan versi ESP-IDF 5.3 untuk memastikan kestabilan fitur.

🔧 Instalasi dan Konfigurasi Lingkungan

Instal ESP toolchain dan library pendukung:

cargo install espup
cargo install espflash


Pastikan Rust dapat mendeteksi board ESP32:

espflash board-info


Konfigurasi proyek:
Tambahkan dependensi yang diperlukan pada Cargo.toml, seperti:

Library untuk WiFi

Modul InfluxDB

Modul UART RS485

Modul sensor SHT20

Modul relay dan buzzer

💻 Pemrograman main.rs

File main.rs berisi:

Inisialisasi WiFi

Koneksi ke InfluxDB lokal

Integrasi dengan ThingsBoard (IoT Platform)

Pembacaan data sensor melalui RS485

Kontrol relay dan buzzer sebagai aktuator

📡 Koneksi ke InfluxDB Lokal

Agar ESP32 dapat mengirim data ke InfluxDB, pastikan:

Anda sudah memiliki ORG ID, Bucket, dan Token dari InfluxDB.

IP laptop (server) dan ESP32 berada pada subnet yang sama.
Misalnya:

Laptop: 192.168.1.10

ESP32: 192.168.1.20

Keduanya harus terhubung ke WiFi yang sama agar komunikasi berhasil.

Analogi subnet:

Subnet bisa diibaratkan seperti satu komplek perumahan (WiFi).
Laptop dan ESP32 harus berada di komplek yang sama agar bisa berkomunikasi.
Namun, alamat rumahnya (IP) harus berbeda agar tidak saling bertabrakan.

☁️ Integrasi dengan ThingsBoard

Buka demo.thingsboard.io
.

Aktifkan device dan dapatkan access token.

Masukkan token atau kredensial MQTT (username dan password) ke dalam kode main.rs.

Jalankan program — data sensor akan otomatis tampil di dashboard ThingsBoard.

🔌 Fitur Utama

✅ Pembacaan data sensor suhu & kelembapan (SHT20)
✅ Komunikasi data melalui RS485 (MAX485)
✅ Pengiriman data ke InfluxDB lokal
✅ Monitoring online melalui ThingsBoard IoT
✅ Kontrol aktuator (Relay & Buzzer)

👥 Tim Kelompok 7

Mata Kuliah Sistem Kontrol Terdistribusi (SKT)
Jurusan Teknik — 2025
