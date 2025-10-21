// main.rs (full revisi, tipe pointer disesuaikan untuk esp-idf-sys yang mengharapkan *const u8)
extern crate alloc;

use anyhow::{bail, Context, Result};
use log::{error, info, warn};

use std::thread; // <<— tambahan
use std::ptr;

use core::ffi::c_char;

use esp_idf_sys as sys; // C-API (MQTT & HTTP)

use alloc::ffi::CString;
use alloc::string::String;
use alloc::string::ToString;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration as StaCfg, Configuration as WifiCfg, EspWifi},
};

use esp_idf_svc::hal::{
    delay::FreeRtos,
    gpio::AnyIOPin,
    peripherals::Peripherals,
    uart::{config::Config as UartConfig, UartDriver},
    units::Hertz,
};

use serde_json::json;

// =================== KONFIGURASI ===================
// Wi-Fi
const WIFI_SSID: &str = "portal0";
const WIFI_PASS: &str = "00012345";

// ThingsBoard Cloud (MQTT Basic)
const TB_MQTT_URL: &str = "mqtt://demo.thingsboard.io:1883";
const TB_CLIENT_ID: &str = "esp32";
const TB_USERNAME: &str = "projekskt";
const TB_PASSWORD: &str = "12345678";

// InfluxDB (contoh lokal dulu biar gampang debug TLS)
// ================= InfluxDB Configuration =================
const INFLUX_URL:   &str = "http://10.206.197.164:8086"; // alamat server InfluxDB kamu
const INFLUX_ORG_ID: &str = "c434171d7ad1fa7b";                // nama org (bisa org name atau orgID UUID)
const INFLUX_BUCKET: &str = "yusron";                  // nama bucket
const INFLUX_TOKEN:  &str = "mBwSBuPiIrf4eDW6uUq7OLbV1TwnSkM-3Ihp3is9jNMjUEzPHmIitXOHoKudGtGTg0-T8WFy4sCKjao_sx4EeQ=="; // token tulis Influx


// Modbus (SHT20 via RS485)
const MODBUS_ID: u8 = 0x01;
const BAUD: u32 = 9_600;

// UART selection & pins: Ubah sesuai board Anda
// Saya menggunakan UART1 contoh pin TX=GPIO17, RX=GPIO18 di sini — GANTI jika board Anda berbeda
const UART_PORT_STR: &str = "UART1";
// NOTE: ubah saja nilai pin di bagian "let tx = pins.gpio17;" kalau board Anda pakai pin lain

// =================== Util ===================
#[inline(always)]
fn ms_to_ticks(ms: u32) -> u32 {
    (ms as u64 * { sys::configTICK_RATE_HZ as u64 } / 1000) as u32
}

fn looks_like_uuid(s: &str) -> bool {
    s.len() == 36 && s.matches('-').count() == 4
}

// Minimal percent-encoding untuk komponen query (RFC 3986 unreserved: ALNUM -_.~)
fn url_encode_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.as_bytes() {
        let c = *b as char;
        if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
            out.push(c);
        } else {
            // %HH (pastikan gunakan nilai byte)
            let _ = core::fmt::write(&mut out, format_args!("%{:02X}", *b));
        }
    }
    out
}

// =================== MQTT client (C-API) ===================
// SimpleMqttClient menyimpan pointer CString yang dialihkan dengan into_raw()
// agar pointer tetap valid sampai Drop dipanggil.
struct SimpleMqttClient {
    client: *mut sys::esp_mqtt_client,
    broker_uri: *mut c_char,
    username: *mut c_char,
    password: *mut c_char,
    client_id: *mut c_char,
}

impl SimpleMqttClient {
    fn new(broker_url: &str, username: &str, password: &str, client_id: &str) -> Result<Self> {
        unsafe {
            // alloakati CString dan lepaskan ke raw pointer sehingga pointer tetap
            // valid sampai kita panggil CString::from_raw()
            let broker_uri = CString::new(broker_url)?.into_raw();
            let username_p = CString::new(username)?.into_raw();
            let password_p = CString::new(password)?.into_raw();
            let client_id_p = CString::new(client_id)?.into_raw();

            let mut cfg: sys::esp_mqtt_client_config_t = core::mem::zeroed();
            // cast pointer sesuai ekspektasi C-API (binding esp-idf-sys di targetmu mengharapkan *const u8)
            cfg.broker.address.uri = broker_uri as *const u8;
            cfg.credentials.username = username_p as *const u8;
            cfg.credentials.client_id = client_id_p as *const u8;
            cfg.credentials.authentication.password = password_p as *const u8;
            cfg.session.keepalive = 30;
            cfg.network.timeout_ms = 20_000;

            let client = sys::esp_mqtt_client_init(&cfg);
            if client.is_null() {
                // cleanup pointer jika gagal
                let _ = CString::from_raw(broker_uri);
                let _ = CString::from_raw(username_p);
                let _ = CString::from_raw(password_p);
                let _ = CString::from_raw(client_id_p);
                bail!("Failed to initialize MQTT client");
            }
            let err = sys::esp_mqtt_client_start(client);
            if err != sys::ESP_OK {
                // cleanup dan destroy client
                sys::esp_mqtt_client_destroy(client);
                let _ = CString::from_raw(broker_uri);
                let _ = CString::from_raw(username_p);
                let _ = CString::from_raw(password_p);
                let _ = CString::from_raw(client_id_p);
                bail!("Failed to start MQTT client, esp_err=0x{:X}", err as u32);
            }
            sys::vTaskDelay(ms_to_ticks(2500));
            Ok(Self {
                client,
                broker_uri,
                username: username_p,
                password: password_p,
                client_id: client_id_p,
            })
        }
    }

    fn publish(&self, topic: &str, data: &str) -> Result<()> {
        unsafe {
            let topic_c = CString::new(topic)?;
            // esp_mqtt_client_publish signature di bindingmu mengharapkan *const u8
            let msg_id = sys::esp_mqtt_client_publish(
                self.client,
                topic_c.as_ptr() as *const u8,
                data.as_ptr(),
                data.len() as i32,
                1,
                0,
            );
            if msg_id < 0 {
                bail!("Failed to publish message, code: {}", msg_id);
            }
            info!("MQTT published (id={})", msg_id);
            Ok(())
        }
    }
}

impl Drop for SimpleMqttClient {
    fn drop(&mut self) {
        unsafe {
            if !self.client.is_null() {
                sys::esp_mqtt_client_stop(self.client);
                sys::esp_mqtt_client_destroy(self.client);
                self.client = ptr::null_mut();
            }
            if !self.broker_uri.is_null() {
                let _ = CString::from_raw(self.broker_uri);
                self.broker_uri = ptr::null_mut();
            }
            if !self.username.is_null() {
                let _ = CString::from_raw(self.username);
                self.username = ptr::null_mut();
            }
            if !self.password.is_null() {
                let _ = CString::from_raw(self.password);
                self.password = ptr::null_mut();
            }
            if !self.client_id.is_null() {
                let _ = CString::from_raw(self.client_id);
                self.client_id = ptr::null_mut();
            }
        }
    }
}

// =================== CRC & Modbus util ===================
fn crc16_modbus(mut crc: u16, byte: u8) -> u16 {
    crc ^= byte as u16;
    for _ in 0..8 {
        crc = if (crc & 1) != 0 { (crc >> 1) ^ 0xA001 } else { crc >> 1 };
    }
    crc
}
fn modbus_crc(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data { crc = crc16_modbus(crc, b); }
    crc
}
fn build_read_req(slave: u8, func: u8, start_reg: u16, qty: u16) -> heapless::Vec<u8, 256> {
    use heapless::Vec;
    let mut pdu: Vec<u8, 256> = Vec::new();
    pdu.push(slave).unwrap();
    pdu.push(func).unwrap();
    pdu.push((start_reg >> 8) as u8).unwrap();
    pdu.push((start_reg & 0xFF) as u8).unwrap();
    pdu.push((qty >> 8) as u8).unwrap();
    pdu.push((qty & 0xFF) as u8).unwrap();
    let crc = modbus_crc(&pdu);
    pdu.push((crc & 0xFF) as u8).unwrap();
    pdu.push((crc >> 8) as u8).unwrap();
    pdu
}
fn parse_read_resp(expected_slave: u8, qty: u16, buf: &[u8]) -> Result<heapless::Vec<u16, 64>> {
    use heapless::Vec;
    if buf.len() >= 5 && (buf[1] & 0x80) != 0 {
        let crc_rx = u16::from(buf[4]) << 8 | u16::from(buf[3]);
        let crc_calc = modbus_crc(&buf[..3]);
        if crc_rx == crc_calc {
            let code = buf[2];
            bail!("Modbus exception 0x{:02X}", code);
        } else {
            bail!("Exception frame CRC mismatch");
        }
    }
    let need = 1 + 1 + 1 + (2 * qty as usize) + 2;
    if buf.len() < need { bail!("Response too short: got {}, need {}", buf.len(), need); }
    if buf[0] != expected_slave { bail!("Unexpected slave id: got {}, expected {}", buf[0], expected_slave); }
    if buf[1] != 0x03 && buf[1] != 0x04 { bail!("Unexpected function code: 0x{:02X}", buf[1]); }
    let bc = buf[2] as usize;
    if bc != 2 * qty as usize { bail!("Unexpected byte count: {}", bc); }
    let crc_rx = u16::from(buf[need - 1]) << 8 | u16::from(buf[need - 2]);
    let crc_calc = modbus_crc(&buf[..need - 2]);
    if crc_rx != crc_calc { bail!("CRC mismatch: rx=0x{:04X}, calc=0x{:04X}", crc_rx, crc_calc); }

    let mut out: Vec<u16, 64> = Vec::new();
    for i in 0..qty as usize {
        let hi = buf[3 + 2 * i] as u16;
        let lo = buf[3 + 2 * i + 1] as u16;
        out.push((hi << 8) | lo).unwrap();
    }
    Ok(out)
}

// =================== RS485 helpers ===================
fn rs485_write(uart: &UartDriver<'_>, data: &[u8]) -> Result<()> {
    uart.write(data)?;
    uart.wait_tx_done(300)?;
    FreeRtos::delay_ms(5);
    Ok(())
}
fn rs485_read(uart: &UartDriver<'_>, dst: &mut [u8], ticks: u32) -> Result<usize> {
    let n = uart.read(dst, ticks)?;
    use core::fmt::Write;
    let mut s = String::new();
    for b in &dst[..n] { write!(&mut s, "{:02X} ", b).ok(); }
    info!("RS485 RX {} bytes: {}", n, s);
    Ok(n)
}
fn try_read(
    uart: &UartDriver<'_>,
    func: u8, start: u16, qty: u16, ticks: u32,
) -> Result<heapless::Vec<u16, 64>> {
    uart.clear_rx()?;
    let req = build_read_req(MODBUS_ID, func, start, qty);
    rs485_write(uart, &req)?;    // tanpa de (jika perlu, atur DE pin di luar fungsi ini)
    let mut buf = [0u8; 64];
    let n = rs485_read(uart, &mut buf, ticks)?;
    parse_read_resp(MODBUS_ID, qty, &buf[..n])
}

fn probe_map(
    uart: &UartDriver<'_>,
) -> Option<(u8, u16, u16)> {
    for &fc in &[0x04u8, 0x03u8] {
        for start in 0x0000u16..=0x0010u16 {
            for &qty in &[1u16, 2u16] {
                if let Ok(regs) = try_read(uart, fc, start, qty, 250) {
                    info!("FOUND: fc=0x{:02X}, start=0x{:04X}, qty={}, regs={:04X?}",
                          fc, start, qty, regs.as_slice());
                    return Some((fc, start, qty));
                }
            }
        }
    }
    None
}
fn read_sht20_with_map(
    uart: &UartDriver<'_>,
    fc: u8, start: u16, qty: u16,
) -> Result<(f32, f32)> {
    // Coba baca sesuai map awal
    let regs = try_read(uart, fc, start, qty, 300)?;
    // Jika kita sudah dapat 2 register, langsung ambil
    if regs.len() >= 2 {
        let raw_t = regs[0];
        let raw_h = regs[1];
        let temp_c = (raw_t as f32) * 0.1;
        let rh_pct = (raw_h as f32) * 0.1;
        return Ok((temp_c, rh_pct));
    }

    // Jika hanya 1 register dikembalikan, coba baca register berikutnya terpisah
    if regs.len() == 1 {
        let raw_t = regs[0];
        // coba read start+1 sebanyak 1 register
        match try_read(uart, fc, start.wrapping_add(1), 1, 300) {
            Ok(regs2) => {
                if regs2.len() >= 1 {
                    let raw_h = regs2[0];
                    let temp_c = (raw_t as f32) * 0.1;
                    let rh_pct = (raw_h as f32) * 0.1;
                    return Ok((temp_c, rh_pct));
                } else {
                    // tidak ada register ke-2
                    let temp_c = (raw_t as f32) * 0.1;
                    return Ok((temp_c, 0.0));
                }
            }
            Err(e) => {
                // gagal baca register kedua; laporkan tapi tetap kembalikan suhu
                warn!("Failed to read humidity register separately: {:?}", e);
                let temp_c = (raw_t as f32) * 0.1;
                return Ok((temp_c, 0.0));
            }
        }
    }

    // Kalau tidak ada register sama sekali (harusnya tidak terjadi)
    bail!("No registers returned from device");
}


// =================== Wi-Fi (BlockingWifi) ===================
fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> Result<()> {
    let cfg = WifiCfg::Client(StaCfg {
        ssid: heapless::String::try_from(WIFI_SSID).unwrap(),
        password: heapless::String::try_from(WIFI_PASS).unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        channel: None, // biarkan scan
        ..Default::default()
    });
    wifi.set_configuration(&cfg)?;
    wifi.start()?;
    info!("Wi-Fi driver started");

    if let Err(e) = wifi.connect() {
        warn!("Wi-Fi connect() error: {e:?}; lanjut offline");
        return Ok(());
    }
    match wifi.wait_netif_up() {
        Ok(()) => {
            let ip = wifi.wifi().sta_netif().get_ip_info()?;
            info!("✅ Wi-Fi connected. IP = {}", ip.ip);
        }
        Err(e) => {
            warn!("⚠ wait_netif_up timeout: {e:?}; lanjut offline");
            let _ = wifi.disconnect();
            let _ = wifi.stop();
        }
    }
    Ok(())
}

// =================== Influx helpers ===================
// Perbaikan: Influx LP tanpa sufiks 'f'
fn influx_line(measurement: &str, device: &str, t_c: f32, h_pct: f32) -> String {
    format!(
        "{},device={} temperature_c={},humidity_pct={}",
        measurement, device, t_c, h_pct
    )
}

// =================== Influx helpers (HTTP via esp_http_client) ===================
// Mengelola CString dengan into_raw() dan memastikan from_raw() dipanggil setelah cleanup.
// ganti fungsi influx_write lama dengan ini
fn influx_write(lp: &str) -> Result<()> {
    unsafe {
        // sesuaikan org query
        let org_q = if looks_like_uuid(INFLUX_ORG_ID) { "orgID" } else { "org" };
        let url = format!(
            "{}/api/v2/write?{}={}&bucket={}&precision=ms",
            INFLUX_URL,
            org_q,
            url_encode_component(INFLUX_ORG_ID),
            url_encode_component(INFLUX_BUCKET)
        );

        // buat raw CString untuk url/header (harus dibebaskan kembali)
        let url_p = CString::new(url.clone()).context("CString::new url")?.into_raw();

        // konfigurasi dasar client (beri sedikit timeout yang lebih besar jika tersedia)
        let mut cfg: sys::esp_http_client_config_t = core::mem::zeroed();
        cfg.url = url_p as *const u8;
        cfg.method = sys::esp_http_client_method_t_HTTP_METHOD_POST;
        // jika binding/versimu punya field timeout_ms, bisa di-set di sini:
        // cfg.timeout_ms = 20_000; // <- uncomment kalau tersedia

        // retry loop sederhana
        let mut last_err: Option<String> = None;
        for attempt in 1..=3 {
            let client = sys::esp_http_client_init(&cfg);
            if client.is_null() {
                last_err = Some("esp_http_client_init returned NULL".to_string());
                break;
            }

            // prepare headers (raw)
            let h_auth = CString::new("Authorization").unwrap().into_raw();
            let v_auth = CString::new(format!("Token {}", INFLUX_TOKEN)).unwrap().into_raw();
            let h_ct   = CString::new("Content-Type").unwrap().into_raw();
            let v_ct   = CString::new("text/plain").unwrap().into_raw();

            sys::esp_http_client_set_header(client, h_auth as *const u8, v_auth as *const u8);
            sys::esp_http_client_set_header(client, h_ct   as *const u8, v_ct   as *const u8);

            // set body
            sys::esp_http_client_set_post_field(client, lp.as_ptr(), lp.len() as i32);

            info!("Influx write attempt {}/3 -> URL={}", attempt, url);

            let err = sys::esp_http_client_perform(client);
            if err == sys::ESP_OK {
                let status = sys::esp_http_client_get_status_code(client);
                if status == 204 {
                    // sukses
                    sys::esp_http_client_cleanup(client);
                    // free raw CStrings
                    let _ = CString::from_raw(url_p);
                    let _ = CString::from_raw(h_auth);
                    let _ = CString::from_raw(v_auth);
                    let _ = CString::from_raw(h_ct);
                    let _ = CString::from_raw(v_ct);
                    return Ok(());
                } else {
                    warn!("Influx write HTTP status {}", status);
                    last_err = Some(format!("HTTP status {}", status));
                }
            } else {
                // log numeric code dan pesan singkat
                last_err = Some(format!("esp_http_client_perform failed: 0x{:X}", err as u32));
                warn!("esp_http_client_perform returned 0x{:X}", err as u32);
            }

            // cleanup client & headers, kemudian tunggu sebelum retry
            sys::esp_http_client_cleanup(client);
            let _ = CString::from_raw(h_auth);
            let _ = CString::from_raw(v_auth);
            let _ = CString::from_raw(h_ct);
            let _ = CString::from_raw(v_ct);

            // tunda (FreeRTOS ticks): 500 ms
            FreeRtos::delay_ms(500 * attempt as u32);
        }

        // free url if we exit loop without success
        let _ = CString::from_raw(url_p);

        if let Some(e) = last_err {
            bail!(e);
        } else {
            bail!("Unknown error writing to Influx");
        }
    }
}



// =================== ENTRYPOINT: Cara Kedua ===================

fn main() -> Result<()> {
    // Patch & logger ringan saja di task main
    sys::link_patches();
    EspLogger::initialize_default();
    info!("▶ starting user thread with big stack ...");

    // Jalankan seluruh logic di thread khusus dengan stack besar
    thread::Builder::new()
        .name("app".into())
        .stack_size(128 * 1024) // naikkan sementara untuk stabilitas
        .spawn(|| {
            if let Err(e) = app_main() {
                error!("app_main error: {e:?}");
            }
        })?;

    // Jangan biarkan task main exit
    loop { FreeRtos::delay_ms(1000); }
}

// Seluruh isi main() lama dipindahkan ke sini
fn app_main() -> Result<()> {
    info!("🚀 Modbus RS485 + ThingsBoard MQTT Basic + InfluxDB (revisi)");

    // Peripherals & services
    let peripherals = Peripherals::take().context("Peripherals::take")?;
    let pins = peripherals.pins;
    let sys_loop = EspSystemEventLoop::take().context("eventloop")?;
    let nvs = EspDefaultNvsPartition::take().context("nvs")?;

    // Wi-Fi via BlockingWifi
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;
    connect_wifi(&mut wifi)?;

    // MQTT ThingsBoard (Basic)
    let mqtt = SimpleMqttClient::new(TB_MQTT_URL, TB_USERNAME, TB_PASSWORD, TB_CLIENT_ID)?;
    info!("MQTT connected to {}", TB_MQTT_URL);

    // ====== RS485: NOTE — gunakan UART1 atau UART2, hindari UART0 yang dipakai console ======
    info!("checkpoint: sebelum inisialisasi UART ({})", UART_PORT_STR);

    // Pilih pin TX/RX — ubah `pins.gpio17` / `pins.gpio18` kalau board Anda pakai pin lain
    let tx = pins.gpio17; // contoh: ubah jika perlu
    let rx = pins.gpio18; // contoh: ubah jika perlu

    let cfg = UartConfig::new().baudrate(Hertz(BAUD));

    // Inisialisasi UART secara eksplisit dan tangkap error untuk logging
    let uart = match UartDriver::new(peripherals.uart1, tx, rx, None::<AnyIOPin>, None::<AnyIOPin>, &cfg) {
        Ok(u) => {
            info!("{} init OK (TX=GPIO17, RX=GPIO18) — pastikan pin sesuai board Anda", UART_PORT_STR);
            u
        }
        Err(e) => {
            error!("{} init FAILED: {:?} — periksa pin/konflik dengan console", UART_PORT_STR, e);
            return Err(e).context("UartDriver::new failed");
        }
    };

    info!("checkpoint: setelah inisialisasi UART");
// --- MULAI SNIPPET RELAY (paste DI SINI, setelah `info!("checkpoint: setelah inisialisasi UART");`) ---
let relay_pin = pins.gpio16; // GANTI dengan pin yang kamu pakai untuk IN1

// --- REPLACE bagian yang memanggil `into_output()` dengan yang ini ---
thread::Builder::new()
    .name("relay".into())
    .stack_size(8 * 1024)
    .spawn(move || {
        // Buat PinDriver sebagai output (mengembalikan Result)
        match esp_idf_hal::gpio::PinDriver::output(relay_pin) {
            Ok(mut p) => {
                info!("Relay thread started on gpio16 (IN1)");
                loop {
                    // Untuk modul active-LOW: set_low() -> RELAY ON, set_high() -> RELAY OFF
                    // set_low()/set_high() kembalikan Result, gunakan .ok() agar tidak panik
                    let _ = p.set_low();
                    FreeRtos::delay_ms(2000);
                    let _ = p.set_high();
                    FreeRtos::delay_ms(2000);
                }
            }
            Err(e) => {
                error!("Relay PinDriver::output failed: {:?}", e);
                loop { FreeRtos::delay_ms(1000); }
            }
        }
    })?;

// --- AKHIR SNIPPET RELAY ---
// --- MULAI SNIPPET BUZZER (paste DI SINI, setelah `info!("checkpoint: setelah inisialisasi UART");`) ---
// GANTI dengan pin yang ingin kamu pakai untuk SIG buzzer, contoh gpio15
let buzzer_pin = pins.gpio15;

thread::Builder::new()
    .name("buzzer".into())
    .stack_size(8 * 1024)
    .spawn(move || {
        match esp_idf_hal::gpio::PinDriver::output(buzzer_pin) {
            Ok(mut bz) => {
                info!("Buzzer thread started on gpio15");
                loop {
                    // Demo: beep cepat 200ms setiap 5 detik
                    let _ = bz.set_high(); // aktifkan buzzer (cek polarity modul; ganti kalau perlu)
                    FreeRtos::delay_ms(200);
                    let _ = bz.set_low();  // matikan
                    FreeRtos::delay_ms(4800);
                }
            }
            Err(e) => {
                error!("Buzzer PinDriver::output failed: {:?}", e);
                loop { FreeRtos::delay_ms(1000); }
            }
        }
    })?;
// --- AKHIR SNIPPET BUZZER ---

    // Probe mapping registri SHT20 (opsional)
    let (mut fc_use, mut start_use, mut qty_use) = (0x04u8, 0x0000u16, 2u16);
    if let Some((fc, start, qty)) = probe_map(&uart) {
        (fc_use, start_use, qty_use) = (fc, start, qty);
        info!("Using map: fc=0x{:02X}, start=0x{:04X}, qty={}", fc_use, start_use, qty_use);
    } else {
        warn!("Probe failed. Fallback map: fc=0x{:02X}, start=0x{:04X}, qty={}", fc_use, start_use, qty_use);
    }

    info!("checkpoint: sebelum loop baca main");

    // Loop baca + publish + write Influx
    let topic_tele = "v1/devices/me/telemetry";
    loop {
        match read_sht20_with_map(&uart, fc_use, start_use, qty_use) {
            Ok((t, h)) => {
                // ThingsBoard telemetry JSON
                let ts_ms = unsafe { sys::esp_timer_get_time() } / 1000;
                let payload = json!({
                    "sensor":"sht20",
                    "temperature_c": (t * 10.0).round()/10.0,
                    "humidity_pct": (h * 10.0).round()/10.0,
                    "ts_ms": ts_ms
                }).to_string();

                println!("{}", payload);

                if let Err(e) = mqtt.publish(topic_tele, &payload) {
                    error!("MQTT publish error: {e:?}");
                }

                // Influx Line Protocol
                let lp = influx_line("sht20", TB_CLIENT_ID, (t * 10.0).round()/10.0, (h * 10.0).round()/10.0);
                if let Err(e) = influx_write(&lp) {
                    warn!("Influx write failed: {e}");
                }
            }
            Err(e) => {
                error!("Modbus read error: {e:?}");
            }
        }
        FreeRtos::delay_ms(1000);
    }
}
