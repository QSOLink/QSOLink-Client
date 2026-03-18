use crate::models::Contact;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_adif(contacts: &[Contact], path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    file.write_all(b"<ADIF_VERS:5>2.0.0\n")?;
    file.write_all(b"<PROGRAMID:5>QSOLink\n")?;
    file.write_all(b"<EOH>\n\n")?;

    for contact in contacts {
        let adif = contact_to_adif(contact);
        file.write_all(adif.as_bytes())?;
        file.write_all(b"<EOR>\n")?;
    }

    Ok(())
}

fn contact_to_adif(contact: &Contact) -> String {
    let mut parts = Vec::new();

    if !contact.call_sign.is_empty() {
        parts.push(format!(
            "<CALL:{}>{}",
            contact.call_sign.len(),
            contact.call_sign
        ));
    }

    if !contact.name.is_empty() {
        parts.push(format!("<NAME:{}>{}", contact.name.len(), contact.name));
    }

    if !contact.qth.is_empty() {
        parts.push(format!("<QTH:{}>{}", contact.qth.len(), contact.qth));
    }

    if contact.frequency > 0.0 {
        let freq_str = format!("{:.3}", contact.frequency);
        parts.push(format!("<FREQ:{}>{}", freq_str.len(), freq_str));
    }

    if !contact.band.is_empty() {
        parts.push(format!("<BAND:{}>{}", contact.band.len(), contact.band));
    }

    if !contact.mode.is_empty() {
        parts.push(format!("<MODE:{}>{}", contact.mode.len(), contact.mode));
    }

    if !contact.rst_sent.is_empty() {
        parts.push(format!(
            "<RST_SENT:{}>{}",
            contact.rst_sent.len(),
            contact.rst_sent
        ));
    }

    if !contact.rst_recv.is_empty() {
        parts.push(format!(
            "<RST_RCVD:{}>{}",
            contact.rst_recv.len(),
            contact.rst_recv
        ));
    }

    if !contact.notes.is_empty() {
        parts.push(format!(
            "<COMMENT:{}>{}",
            contact.notes.len(),
            contact.notes
        ));
    }

    if !contact.qso_date.is_empty() {
        let date_clean = contact.qso_date.replace("-", "");
        parts.push(format!("<QSO_DATE:8>{}", date_clean));
    }

    if !contact.qso_time.is_empty() {
        let time_clean = format!("{:0<4}", contact.qso_time.replace(":", ""));
        parts.push(format!("<TIME_ON:4>{}", time_clean));
    }

    parts.join("\n")
}

pub fn generate_default_filename() -> String {
    let now = chrono::Local::now();
    format!("qso_export_{}.adi", now.format("%Y%m%d_%H%M"))
}

pub struct CabrilloConfig {
    pub callsign: String,
    pub contest: String,
    pub category_operator: String,
    pub category_transmitter: String,
    pub category_power: String,
    pub category_station: String,
    pub category_time: String,
    pub club: String,
    pub operators: String,
    pub name: String,
    pub address: String,
    pub address_city: String,
    pub address_state: String,
    pub address_zip: String,
    pub address_country: String,
    pub email: String,
    pub grid_square: String,
}

impl Default for CabrilloConfig {
    fn default() -> Self {
        Self {
            callsign: String::new(),
            contest: "CQ-WW-SSB".to_string(),
            category_operator: "SINGLE-OP".to_string(),
            category_transmitter: "ONE".to_string(),
            category_power: "HIGH".to_string(),
            category_station: "FIXED".to_string(),
            category_time: "24-HOURS".to_string(),
            club: String::new(),
            operators: String::new(),
            name: String::new(),
            address: String::new(),
            address_city: String::new(),
            address_state: String::new(),
            address_zip: String::new(),
            address_country: String::new(),
            email: String::new(),
            grid_square: String::new(),
        }
    }
}

pub fn export_cabrillo(
    contacts: &[Contact],
    path: &Path,
    config: &CabrilloConfig,
) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    file.write_all(b"START-OF-LOG: 3.0\n")?;
    file.write_all(format!("CALLSIGN: {}\n", config.callsign).as_bytes())?;
    file.write_all(format!("CONTEST: {}\n", config.contest).as_bytes())?;
    file.write_all(format!("CATEGORY-OPERATOR: {}\n", config.category_operator).as_bytes())?;
    file.write_all(format!("CATEGORY-TRANSMITTER: {}\n", config.category_transmitter).as_bytes())?;
    file.write_all(format!("CATEGORY-POWER: {}\n", config.category_power).as_bytes())?;
    file.write_all(format!("CATEGORY-STATION: {}\n", config.category_station).as_bytes())?;
    file.write_all(format!("CATEGORY-TIME: {}\n", config.category_time).as_bytes())?;

    if !config.club.is_empty() {
        file.write_all(format!("CLUB: {}\n", config.club).as_bytes())?;
    }
    if !config.operators.is_empty() {
        file.write_all(format!("OPERATORS: {}\n", config.operators).as_bytes())?;
    }
    if !config.name.is_empty() {
        file.write_all(format!("NAME: {}\n", config.name).as_bytes())?;
    }
    if !config.address.is_empty() {
        file.write_all(format!("ADDRESS: {}\n", config.address).as_bytes())?;
    }
    if !config.address_city.is_empty() {
        file.write_all(format!("ADDRESS-CITY: {}\n", config.address_city).as_bytes())?;
    }
    if !config.address_state.is_empty() {
        file.write_all(format!("ADDRESS-STATE: {}\n", config.address_state).as_bytes())?;
    }
    if !config.address_zip.is_empty() {
        file.write_all(format!("ADDRESS-ZIP: {}\n", config.address_zip).as_bytes())?;
    }
    if !config.address_country.is_empty() {
        file.write_all(format!("ADDRESS-COUNTRY: {}\n", config.address_country).as_bytes())?;
    }
    if !config.email.is_empty() {
        file.write_all(format!("EMAIL: {}\n", config.email).as_bytes())?;
    }
    if !config.grid_square.is_empty() {
        file.write_all(format!("GRID-LOCATOR: {}\n", config.grid_square).as_bytes())?;
    }

    file.write_all(b"SOAPY:\n")?;
    file.write_all(b"CREATED-BY: QSOLink\n")?;
    file.write_all(b"\n")?;

    for contact in contacts {
        let cabrillo_line = contact_to_cabrillo(contact);
        file.write_all(cabrillo_line.as_bytes())?;
        file.write_all(b"\n")?;
    }

    file.write_all(b"END-OF-LOG:\n")?;

    Ok(())
}

fn contact_to_cabrillo(contact: &Contact) -> String {
    let freq_khz = (contact.frequency * 1000.0) as u32;
    let mode_code = mode_to_cabrillo(&contact.mode);
    let qso_date = contact.qso_date.replace("-", "");
    let qso_time = format!("{:0<4}", contact.qso_time.replace(":", ""));

    format!(
        "QSO: {} {} {} {} {} {} {}",
        freq_khz,
        mode_code,
        qso_date,
        qso_time,
        pad_field(&contact.call_sign, 14),
        pad_field(&contact.rst_sent, 3),
        pad_field(&contact.rst_recv, 3)
    )
}

fn mode_to_cabrillo(mode: &str) -> u8 {
    match mode.to_uppercase().as_str() {
        "CW" => 1,
        "PHONE" | "SSB" | "FM" | "AM" => 2,
        "DIG" | "DATA" | "RTTY" | "PSK31" | "PSK63" | "FT8" | "FT4" => 3,
        _ => 2,
    }
}

fn pad_field(field: &str, width: usize) -> String {
    if field.len() >= width {
        field[..width].to_string()
    } else {
        format!("{:width$}", field, width = width)
    }
}

pub fn generate_cabrillo_filename() -> String {
    let now = chrono::Local::now();
    format!("qso_export_{}.log", now.format("%Y%m%d_%H%M"))
}
