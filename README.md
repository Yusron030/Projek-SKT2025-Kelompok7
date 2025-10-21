# 🧠 Proyek Sistem Kontrol Terdistribusi - Kelompok 7

## 📘 Deskripsi Proyek
Pada proyek mata kuliah **Sistem Kontrol Terdistribusi**, kami dari **Kelompok 7** membuat sebuah sistem yang memiliki beberapa fitur seperti halnya **DCS (Distributed Control System)**.

Sistem ini dibangun menggunakan **ESP32 S3** yang terhubung dengan beberapa sensor dan aktuator melalui komunikasi **RS485 (Modbus)** dan dikontrol menggunakan bahasa pemrograman **Rust**.

---

## ⚙️ Komponen yang Digunakan
- **ESP32 S3**
- **Sensor SHT20** (Suhu & Kelembapan)
- **MAX485 RS485 TTL**
- **Relay Module**
- **Buzzer**

---

## 💻 Instalasi dan Persiapan

### 1️⃣ Install Template Proyek ESP32 untuk Rust
Jalankan perintah berikut di terminal untuk membuat template proyek ESP32:

```bash
cargo generate --git https://github.com/esp-rs/esp-idf-template.git
Kemudian pilih:

markdown
Copy code
> esp32s3
> esp-idf version: 5.3 (recommended for stability)
2️⃣ Instalasi Tools Pendukung
Agar bahasa Rust dapat digunakan untuk memprogram ESP32, lakukan instalasi berikut:

bash
Copy code
cargo install espup
cargo install espflash
📝 Keterangan:

espup digunakan untuk men-setup toolchain ESP-IDF agar kompatibel dengan Rust.

espflash digunakan untuk mengirim program ke ESP32 via USB.

3️⃣ Struktur Proyek
Setelah semua siap, kamu akan memiliki struktur proyek seperti berikut:

css
Copy code
📂 proyek_esp32
 ┣ 📜 Cargo.toml
 ┣ 📜 main.rs
 ┗ 📂 src/
    ┗ 📜 main.rs
Pada file Cargo.toml, tambahkan dependencies yang dibutuhkan seperti untuk WiFi, InfluxDB, UART, dan Sensor SHT20.

🧠 Pemrograman main.rs
Di dalam file main.rs, kita memprogram:

Konektivitas WiFi

Koneksi ke InfluxDB

Koneksi ke ThingsBoard

Pembacaan sensor melalui RS485

Kontrol aktuator (Relay dan Buzzer)

Contoh struktur dasar program:

rust
Copy code
fn main() {
    // Inisialisasi WiFi
    init_wifi("SSID", "PASSWORD");

    // Koneksi ke InfluxDB lokal
    connect_influxdb("http://192.168.x.x:8086", "ORG_ID", "BUCKET", "TOKEN");

    // Inisialisasi komunikasi UART RS485
    init_rs485();

    // Pembacaan data sensor SHT20
    let data = read_sht20();

    // Kirim data ke InfluxDB dan ThingsBoard
    send_to_influxdb(data);
    send_to_thingsboard(data);

    // Kontrol relay & buzzer
    control_actuator(data);
}
🌐 Koneksi ke InfluxDB Lokal
Agar ESP32 bisa mengirim data ke InfluxDB lokal, pastikan:

Gunakan ORG ID, Bucket, dan Token sesuai konfigurasi InfluxDB.

IP laptop dan ESP32 harus dalam satu subnet jaringan.

Penjelasan Singkat:
🔹 Subnet diibaratkan seperti satu kompleks perumahan (WiFi).
Laptop dan ESP32 harus berada di kompleks yang sama, agar bisa saling berkomunikasi.
Namun alamat rumah (IP address) harus berbeda supaya tidak “tabrakan”.

Contoh konfigurasi:

rust
Copy code
let influx_url = "http://192.168.0.10:8086";
let org = "my_org";
let bucket = "sensor_data";
let token = "my_secret_token";
☁️ Koneksi ke ThingsBoard
Untuk ThingsBoard, langkahnya cukup mudah:

Aktifkan Device di https://demo.thingsboard.io

Dapatkan Access Token dari device tersebut.

Masukkan token tersebut pada program main.rs.

Contoh konfigurasi:

rust
Copy code
let thingsboard_host = "demo.thingsboard.io";
let token = "YOUR_DEVICE_TOKEN";
Kirim data menggunakan MQTT:

rust
Copy code
send_to_thingsboard(thingsboard_host, token, data);
🔔 Aktuator (Relay dan Buzzer)
Setelah sensor terbaca dan data terkirim, ESP32 juga mengontrol Relay dan Buzzer untuk memberikan respon terhadap kondisi tertentu (misalnya suhu tinggi atau kelembapan rendah).

rust
Copy code
if suhu > 35.0 {
    relay_on();
    buzzer_on();
} else {
    relay_off();
    buzzer_off();
}
📊 Hasil dan Monitoring
Data dari sensor ditampilkan di InfluxDB (Grafana Dashboard) secara real-time.

Data juga bisa dimonitor di ThingsBoard Cloud dengan tampilan grafik yang interaktif.

👥 Anggota Kelompok 7
Muhammad Yusron Maskur

[Nama Anggota 2]

[Nama Anggota 3]

[Nama Anggota 4]

🧩 Kesimpulan
Proyek ini berhasil mengimplementasikan konsep Distributed Control System (DCS) sederhana menggunakan ESP32 S3 dan Rust, dengan integrasi InfluxDB lokal dan ThingsBoard Cloud sebagai media monitoring data.
