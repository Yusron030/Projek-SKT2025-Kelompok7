# 📡 Proyek Sistem Kontrol Terdistribusi — Kelompok 7

## Deskripsi Singkat

Pada proyek mata kuliah **Sistem Kontrol Terdistribusi**, Kelompok 7 mengembangkan sebuah sistem yang meniru fungsi dasar **DCS (Distributed Control System)**. Sistem ini menggunakan **ESP32-S3** yang terhubung dengan beberapa sensor dan aktuator. Data sensor dikirim ke **InfluxDB** (lokal) dan **ThingsBoard** (cloud) untuk monitoring.

---

## 🔧 Perangkat Keras

* **ESP32-S3**
* **Sensor SHT20** (suhu & kelembapan)
* **MAX485 (RS485 ↔ TTL)**
* **Relay module**
* **Buzzer**

---

## 🛠️ Perangkat Lunak & Tools

* Bahasa: **Rust**
* Template: `esp-idf-template` (esp-rs)
* Tools: `espup`, `espflash`
* Backend monitoring: **InfluxDB (lokal)**, **ThingsBoard (demo cloud)**

---

## Langkah Persiapan & Instalasi

1. **Buat project template**

```bash
cargo generate --git https://github.com/esp-rs/esp-idf-template.git
# Pilih target: esp32s3
# Pilih esp-idf version: 5.3
```

2. **Pasang tooling Rust untuk ESP32**

```bash
cargo install espup
cargo install espflash
```

3. **Inisialisasi toolchain ESP-IDF**

```bash
# jalankan espup (otomatis mengunduh dan men-setup toolchain esp-idf yang kompatibel)
espup install
```

> Catatan: Ikuti petunjuk `espup` untuk PATH dan variabel lingkungan jika diminta.

---

## Struktur Proyek (contoh)

```
tugas-skt/
├─ Cargo.toml
├─ src/
│  └─ main.rs
└─ README.md
```

Tambahkan dependency di `Cargo.toml` sesuai kebutuhan (WiFi, UART, sensor, MQTT/HTTP client, dsb.).

---

## Contoh Perintah / Bash yang Sering Digunakan

```bash
# Build project
cargo build

# Flash ke ESP32
espflash /dev/ttyACM0 target/xtensa-esp32s3-espidf/release/<nama_bin>

# (Jika menggunakan espflash via cargo)
espflash --release /dev/ttyACM0
```

---

## Panduan Pemrograman (ringkasan `main.rs`)

Pada `main.rs` struktur umumnya meliputi:

1. Inisialisasi hardware (UART/RS485, I2C untuk SHT20, GPIO untuk relay & buzzer)
2. Konektivitas WiFi
3. Koneksi ke InfluxDB (HTTP API)
4. Koneksi ke ThingsBoard (MQTT)
5. Pembacaan sensor melalui RS485 / I2C
6. Logika kontrol aktuator (relay & buzzer)
7. Pengiriman data ke InfluxDB dan ThingsBoard

Contoh pseudocode:

```rust
fn main() {
    init_wifi("SSID", "PASSWORD");
    init_rs485();
    init_i2c();

    loop {
        let sensor = read_sht20();
        let modbus_data = read_rs485();

        send_to_influxdb(&sensor);
        publish_thingsboard(&sensor);

        if sensor.temperature > 35.0 {
            relay_on();
            buzzer_on();
        } else {
            relay_off();
            buzzer_off();
        }

        sleep_ms(1000);
    }
}
```

---

## Konfigurasi InfluxDB (Lokal)

Agar ESP32 dapat mengirim data ke InfluxDB lokal:

* Gunakan **ORG ID**, **Bucket**, dan **Token** sesuai konfigurasi InfluxDB.
* Pastikan **IP laptop (InfluxDB)** dan **ESP32** berada dalam **subnet yang sama** (misal: `192.168.0.x`).
* Pastikan IP berbeda (tidak boleh sama) agar tidak terjadi konflik alamat.

Contoh variabel koneksi di kode:

```rust
let influx_url = "http://10.206.197.164:8086";
let org = "my_org";
let bucket = "sensor_data";
let token = "my_token";
```

---

## Konfigurasi ThingsBoard (demo.thingsboard.io)

1. Buat atau aktifkan device pada `demo.thingsboard.io`.
2. Ambil **Access Token** device.
3. Masukkan token pada konfigurasi MQTT di `main.rs`.

Contoh singkat:

```rust
let thingsboard_host = "demo.thingsboard.io";
let device_token = "DEVICE_TOKEN";
publish_mqtt(thingsboard_host, device_token, payload);
```

---

## Troubleshooting Singkat

* **InfluxDB unreachable**: cek IP, subnet, dan firewall (port 8086).
* **MQTT tidak terkoneksi**: cek token, user/pass, serta koneksi WiFi.
* **RS485 tidak baca sensor**: cek wiring A/B, level shifter, dan konfigurasi UART (baudrate, parity).

---

## Contoh Konfigurasi `Cargo.toml` (placeholder)

```toml
[dependencies]
# contoh dependency, silakan sesuaikan
esp-idf-svc = "*"
esp-idf-hal = "*"
embedded-hal = "*"
# dependency untuk mqtt/http/influx client sesuai kebutuhan
```

---

## Anggota Kelompok

* Adrian Yared Immanuel (2042221080)
* Muhammad Yusron Maskur (2042231030)
* Agus Wedi (2042231066)

---

## Lisensi

Proyek ini dapat di-licence sesuai kesepakatan tim. (Contoh: MIT License)

---

