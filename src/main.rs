use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_6X13_BOLD, ascii::FONT_9X18_BOLD, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay::FreeRtos,
        i2c::{I2cConfig, I2cDriver},
        prelude::*,
    },
    http::client::{Configuration as HttpConfig, EspHttpConnection},
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use std::thread;
use std::time::Duration;

const WIFI_SSID: &str = "SSID";
const WIFI_PASS: &str = "MDP";
const SERVER_URL: &str = "http://my-love.duckdns.org:8080";
const MY_PERSON: &str = "a";
const POLL_INTERVAL: Duration = Duration::from_secs(5);

// ==================== EMOJI BITMAPS 32x32 ====================
// Chaque bitmap = 32x32 pixels = 128 bytes (1 bit par pixel)

// ❤️ Coeur
const HEART: [u8; 128] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xE0, 0x7C, 0x00, 0x07, 0xF0, 0xFE, 0x00,
    0x0F, 0xF9, 0xFF, 0x00, 0x1F, 0xFF, 0xFF, 0x80, 0x1F, 0xFF, 0xFF, 0x80, 0x3F, 0xFF, 0xFF, 0xC0,
    0x3F, 0xFF, 0xFF, 0xC0, 0x3F, 0xFF, 0xFF, 0xC0, 0x3F, 0xFF, 0xFF, 0xC0, 0x3F, 0xFF, 0xFF, 0xC0,
    0x1F, 0xFF, 0xFF, 0x80, 0x1F, 0xFF, 0xFF, 0x80, 0x0F, 0xFF, 0xFF, 0x00, 0x0F, 0xFF, 0xFF, 0x00,
    0x07, 0xFF, 0xFE, 0x00, 0x03, 0xFF, 0xFC, 0x00, 0x01, 0xFF, 0xF8, 0x00, 0x00, 0xFF, 0xF0, 0x00,
    0x00, 0x7F, 0xE0, 0x00, 0x00, 0x3F, 0xC0, 0x00, 0x00, 0x1F, 0x80, 0x00, 0x00, 0x0F, 0x00, 0x00,
    0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// 😘 Bisou (clin d'oeil + coeur)
const KISS: [u8; 128] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x3F, 0xC0, 0x00, 0x00, 0xFF, 0xF0, 0x00, 0x03, 0xFF, 0xFC, 0x00,
    0x07, 0xFF, 0xFE, 0x00, 0x07, 0xFF, 0xFE, 0x00, 0x0F, 0xFF, 0xFF, 0x00, 0x0F, 0xFF, 0xFF, 0x00,
    0x0E, 0x3F, 0xC7, 0x00, 0x0E, 0x3F, 0x87, 0x00, 0x0F, 0x1F, 0x0F, 0x00, 0x0F, 0xFF, 0xFF, 0x00,
    0x0F, 0xFF, 0xFF, 0x60, 0x0F, 0xFF, 0xFF, 0xE0, 0x07, 0xFF, 0xFF, 0xE0, 0x07, 0x81, 0xFE, 0x60,
    0x07, 0x00, 0xFE, 0x00, 0x03, 0xC3, 0xFC, 0x00, 0x00, 0xFF, 0xF0, 0x00, 0x00, 0x3F, 0xC0, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// 🥰 Smiley content
const HAPPY: [u8; 128] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x3F, 0xC0, 0x00, 0x00, 0xFF, 0xF0, 0x00, 0x03, 0xFF, 0xFC, 0x00,
    0x07, 0xFF, 0xFE, 0x00, 0x07, 0xFF, 0xFE, 0x00, 0x0F, 0xFF, 0xFF, 0x00, 0x0F, 0xFF, 0xFF, 0x00,
    0x0E, 0x7F, 0xE7, 0x00, 0x0E, 0x7F, 0xE7, 0x00, 0x0E, 0x7F, 0xE7, 0x00, 0x0F, 0xFF, 0xFF, 0x00,
    0x0F, 0xFF, 0xFF, 0x00, 0x0F, 0xFF, 0xFF, 0x00, 0x07, 0xFF, 0xFE, 0x00, 0x07, 0x80, 0x1E, 0x00,
    0x07, 0x00, 0x0E, 0x00, 0x03, 0xC0, 0x3C, 0x00, 0x00, 0xFF, 0xF0, 0x00, 0x00, 0x3F, 0xC0, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// ⭐ Etoile
const STAR: [u8; 128] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x80, 0x00,
    0x00, 0x03, 0x80, 0x00, 0x00, 0x07, 0xC0, 0x00, 0x00, 0x07, 0xC0, 0x00, 0x00, 0x0F, 0xE0, 0x00,
    0x00, 0x0F, 0xE0, 0x00, 0x1F, 0xFF, 0xFF, 0x00, 0x0F, 0xFF, 0xFE, 0x00, 0x07, 0xFF, 0xFC, 0x00,
    0x03, 0xFF, 0xF8, 0x00, 0x01, 0xFF, 0xF0, 0x00, 0x00, 0xFF, 0xE0, 0x00, 0x01, 0xFF, 0xF0, 0x00,
    0x01, 0xFF, 0xF0, 0x00, 0x03, 0xFF, 0xF8, 0x00, 0x03, 0xEF, 0xF8, 0x00, 0x07, 0xC7, 0xFC, 0x00,
    0x07, 0x83, 0xBC, 0x00, 0x0F, 0x01, 0x1E, 0x00, 0x06, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// 🌙 Lune
const MOON: [u8; 128] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xE0, 0x00, 0x00, 0x1F, 0xE0, 0x00, 0x00, 0x7F, 0xC0, 0x00,
    0x00, 0xFF, 0x80, 0x00, 0x01, 0xFF, 0x00, 0x00, 0x03, 0xFE, 0x00, 0x00, 0x03, 0xFC, 0x00, 0x00,
    0x07, 0xFC, 0x00, 0x00, 0x07, 0xF8, 0x00, 0x00, 0x07, 0xF8, 0x00, 0x00, 0x0F, 0xF8, 0x00, 0x00,
    0x0F, 0xF8, 0x00, 0x00, 0x0F, 0xF8, 0x00, 0x00, 0x0F, 0xF8, 0x00, 0x00, 0x0F, 0xF8, 0x00, 0x00,
    0x07, 0xF8, 0x00, 0x00, 0x07, 0xFC, 0x00, 0x00, 0x07, 0xFC, 0x00, 0x00, 0x03, 0xFE, 0x00, 0x00,
    0x03, 0xFF, 0x00, 0x00, 0x01, 0xFF, 0x80, 0x00, 0x00, 0xFF, 0xC0, 0x00, 0x00, 0x7F, 0xE0, 0x00,
    0x00, 0x1F, 0xF0, 0x00, 0x00, 0x07, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Cherche un bitmap emoji correspondant au contenu
fn get_emoji_bitmap(content: &str) -> Option<&'static [u8; 128]> {
    match content {
        s if s.contains('\u{2764}') || s.contains("❤") => Some(&HEART), // ❤️
        s if s.contains("💕") => Some(&HEART),
        s if s.contains("💜") => Some(&HEART),
        s if s.contains("💋") => Some(&KISS),
        s if s.contains("😘") => Some(&KISS),
        s if s.contains("🥰") => Some(&HAPPY),
        s if s.contains("😍") => Some(&HAPPY),
        s if s.contains("🤗") => Some(&HAPPY),
        s if s.contains("🫶") => Some(&HEART),
        s if s.contains("✨") => Some(&STAR),
        s if s.contains("☀") => Some(&STAR),
        s if s.contains("🌙") => Some(&MOON),
        _ => None,
    }
}

/// Convertit les caracteres accentues en leur version ASCII
fn strip_accents(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            'à' | 'â' | 'ä' | 'á' | 'ã' => output.push('a'),
            'À' | 'Â' | 'Ä' | 'Á' | 'Ã' => output.push('A'),
            'é' | 'è' | 'ê' | 'ë' => output.push('e'),
            'É' | 'È' | 'Ê' | 'Ë' => output.push('E'),
            'î' | 'ï' | 'ì' | 'í' => output.push('i'),
            'Î' | 'Ï' | 'Ì' | 'Í' => output.push('I'),
            'ô' | 'ö' | 'ò' | 'ó' | 'õ' => output.push('o'),
            'Ô' | 'Ö' | 'Ò' | 'Ó' | 'Õ' => output.push('O'),
            'ù' | 'û' | 'ü' | 'ú' => output.push('u'),
            'Ù' | 'Û' | 'Ü' | 'Ú' => output.push('U'),
            'ç' => output.push('c'),
            'Ç' => output.push('C'),
            'ñ' => output.push('n'),
            'Ñ' => output.push('N'),
            'œ' => {
                output.push('o');
                output.push('e');
            }
            'Œ' => {
                output.push('O');
                output.push('E');
            }
            'æ' => {
                output.push('a');
                output.push('e');
            }
            'Æ' => {
                output.push('A');
                output.push('E');
            }
            '\'' | '\u{2019}' => output.push('\''),
            c if c.is_ascii() => output.push(c),
            // Skip les emojis et autres caracteres non-ASCII
            c if c.len_utf8() > 2 => {}
            _ => output.push('?'),
        }
    }
    output
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    FreeRtos::delay_ms(2000);

    let config = I2cConfig::new()
        .baudrate(50.kHz().into())
        .sda_enable_pullup(true)
        .scl_enable_pullup(true);

    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        &config,
    )?;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    let mut init_ok = false;
    for attempt in 1..=5 {
        log::info!("Init SSD1306 tentative {}...", attempt);
        match display.init() {
            Ok(_) => {
                log::info!("SSD1306 OK !");
                init_ok = true;
                break;
            }
            Err(e) => {
                log::warn!("Init fail: {:?}", e);
                FreeRtos::delay_ms(500);
            }
        }
    }
    if !init_ok {
        return Ok(());
    }

    display.clear_buffer();
    draw_text(&mut display, "Love Display", "Demarrage...");
    display.flush().unwrap();

    // ==================== WiFi ====================
    log::info!("Connexion WiFi...");
    display.clear_buffer();
    draw_text(&mut display, "Love Display", "WiFi...");
    display.flush().unwrap();

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.try_into().unwrap(),
        password: WIFI_PASS.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    wifi.start()?;

    for attempt in 1..=5 {
        log::info!("WiFi tentative {}...", attempt);
        display.clear_buffer();
        draw_text(&mut display, "WiFi...", &format!("Tentative {}/5", attempt));
        display.flush().unwrap();

        match wifi.connect() {
            Ok(_) => break,
            Err(e) => {
                log::warn!("WiFi echec: {:?}", e);
                if attempt == 5 {
                    display.clear_buffer();
                    draw_text(&mut display, "WiFi ECHEC", "Reboot...");
                    display.flush().unwrap();
                    thread::sleep(Duration::from_secs(3));
                    unsafe {
                        esp_idf_svc::sys::esp_restart();
                    }
                }
                thread::sleep(Duration::from_secs(2));
            }
        }
    }

    wifi.wait_netif_up()?;
    log::info!("WiFi connecte !");

    // ==================== Boucle principale ====================
    let mut last_timestamp: i64 = 0;

    loop {
        match poll_server(last_timestamp) {
            Ok(Some((content, kind, ts))) => {
                log::info!("Message: {} ({})", content, kind);
                last_timestamp = ts;
                display.clear_buffer();

                if kind == "emoji" {
                    if let Some(bitmap) = get_emoji_bitmap(&content) {
                        // Affiche le bitmap centre (32x32 au milieu de 128x64)
                        let raw: ImageRaw<BinaryColor> = ImageRaw::new(bitmap, 32);
                        let _ = Image::new(&raw, Point::new(48, 16)).draw(&mut display);
                    } else {
                        // Emoji non reconnu, affiche le texte brut
                        let cleaned = strip_accents(&content);
                        draw_big_centered(&mut display, &cleaned);
                    }
                } else {
                    let cleaned = strip_accents(&content);
                    draw_message(&mut display, &cleaned);
                }
                display.flush().unwrap();
            }
            Ok(None) => {}
            Err(e) => {
                log::warn!("Erreur poll: {:?}", e);
            }
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn poll_server(since: i64) -> anyhow::Result<Option<(String, String, i64)>> {
    let url = format!("{}/api/display/{}?since={}", SERVER_URL, MY_PERSON, since);
    let connection = EspHttpConnection::new(&HttpConfig {
        timeout: Some(Duration::from_secs(10)),
        ..Default::default()
    })?;
    let mut client = embedded_svc::http::client::Client::wrap(connection);
    let request = client.get(&url)?;
    let mut response = request.submit()?;
    if response.status() != 200 {
        anyhow::bail!("HTTP {}", response.status());
    }
    let mut buf = [0u8; 1024];
    let mut body = Vec::new();
    loop {
        use embedded_svc::io::Read;
        match response.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => body.extend_from_slice(&buf[..n]),
            Err(e) => {
                log::warn!("Read err: {:?}", e);
                break;
            }
        }
    }
    let json: serde_json::Value = serde_json::from_slice(&body)?;
    if json["has_message"].as_bool() == Some(true) {
        let msg = &json["message"];
        let content = msg["content"].as_str().unwrap_or("?").to_string();
        let kind = msg["kind"].as_str().unwrap_or("text").to_string();
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Ok(Some((content, kind, ts)))
    } else {
        Ok(None)
    }
}

fn draw_text<D: DrawTarget<Color = BinaryColor>>(display: &mut D, line1: &str, line2: &str) {
    let style = MonoTextStyleBuilder::new()
        .font(&FONT_6X13_BOLD)
        .text_color(BinaryColor::On)
        .build();
    let _ = Text::with_alignment(line1, Point::new(64, 24), style, Alignment::Center).draw(display);
    let _ = Text::with_alignment(line2, Point::new(64, 44), style, Alignment::Center).draw(display);
}

fn draw_big_centered<D: DrawTarget<Color = BinaryColor>>(display: &mut D, text: &str) {
    let style = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(BinaryColor::On)
        .build();
    let _ = Text::with_alignment(text, Point::new(64, 38), style, Alignment::Center).draw(display);
}

fn draw_message<D: DrawTarget<Color = BinaryColor>>(display: &mut D, text: &str) {
    let style = MonoTextStyleBuilder::new()
        .font(&FONT_6X13_BOLD)
        .text_color(BinaryColor::On)
        .build();
    let lines: Vec<String> = wrap_text(text, 21);
    let total_height = lines.len() as i32 * 15;
    let start_y = (64 - total_height) / 2 + 12;
    for (i, line) in lines.iter().enumerate() {
        let y = start_y + (i as i32 * 15);
        let _ =
            Text::with_alignment(line, Point::new(64, y), style, Alignment::Center).draw(display);
    }
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.truncate(4);
    lines
}
