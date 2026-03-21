use crate::models::{get_mode_adif_fields, Contact};
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
        let freq_str = format!("{:.5}", contact.frequency);
        parts.push(format!("<FREQ:{}>{}", freq_str.len(), freq_str));
    }

    if !contact.band.is_empty() {
        parts.push(format!("<BAND:{}>{}", contact.band.len(), contact.band));
    }

    if !contact.mode.is_empty() {
        let (adif_mode, adif_submode) = get_mode_adif_fields(&contact.mode);
        parts.push(format!("<MODE:{}>{}", adif_mode.len(), adif_mode));
        if let Some(submode) = adif_submode {
            parts.push(format!("<SUBMODE:{}>{}", submode.len(), submode));
        }
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
        let time_clean = format!("{:0<6}", contact.qso_time.replace(":", ""));
        parts.push(format!("<TIME_ON:6>{}", time_clean));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Contact;

    fn create_test_contact() -> Contact {
        let mut c = Contact::new("W1AW".to_string());
        c.name = "Test Operator".to_string();
        c.qth = "Newington".to_string();
        c.frequency = 14.175;
        c.band = "20m".to_string();
        c.mode = "SSB".to_string();
        c.rst_sent = "59".to_string();
        c.rst_recv = "59".to_string();
        c.notes = "Test QSO".to_string();
        c.qso_date = "2024-01-15".to_string();
        c.qso_time = "120000".to_string();
        c
    }

    #[test]
    fn test_contact_to_adif_basic() {
        let contact = create_test_contact();
        let adif = contact_to_adif(&contact);

        assert!(adif.contains("<CALL:4>W1AW"));
        assert!(adif.contains("<NAME:"));
        assert!(adif.contains("Test Operator"));
        assert!(adif.contains("<QTH:9>Newington"));
        assert!(adif.contains("<BAND:3>20m"));
        assert!(adif.contains("<MODE:"));
        assert!(adif.contains("<RST_SENT:2>59"));
        assert!(adif.contains("<QSO_DATE:8>20240115"));
        assert!(adif.contains("<TIME_ON:6>120000"));
    }

    #[test]
    fn test_contact_to_adif_empty_fields() {
        let c = Contact::new("W1AW".to_string());
        let adif = contact_to_adif(&c);
        assert!(adif.contains("<CALL:4>W1AW"));
        assert!(!adif.contains("<NAME:"));
        assert!(!adif.contains("<QTH:"));
    }

    #[test]
    fn test_contact_to_adif_ft8_mode() {
        let mut c = Contact::new("W1AW".to_string());
        c.mode = "FT8".to_string();
        c.frequency = 14.074;
        c.band = "20m".to_string();
        let adif = contact_to_adif(&c);

        assert!(adif.contains("<MODE:4>DATA"));
        assert!(adif.contains("<SUBMODE:3>FT8"));
    }

    #[test]
    fn test_contact_to_adif_frequency_format() {
        let mut c = Contact::new("W1AW".to_string());
        c.frequency = 14.17500;
        let adif = contact_to_adif(&c);
        assert!(adif.contains("<FREQ:"));
        assert!(adif.contains("14.17500"));
    }

    #[test]
    fn test_mode_to_cabrillo() {
        assert_eq!(mode_to_cabrillo("CW"), 1);
        assert_eq!(mode_to_cabrillo("cw"), 1);
        assert_eq!(mode_to_cabrillo("SSB"), 2);
        assert_eq!(mode_to_cabrillo("PHONE"), 2);
        assert_eq!(mode_to_cabrillo("FM"), 2);
        assert_eq!(mode_to_cabrillo("AM"), 2);
        assert_eq!(mode_to_cabrillo("DIG"), 3);
        assert_eq!(mode_to_cabrillo("FT8"), 3);
        assert_eq!(mode_to_cabrillo("FT4"), 3);
        assert_eq!(mode_to_cabrillo("RTTY"), 3);
        assert_eq!(mode_to_cabrillo("UNKNOWN"), 2);
    }

    #[test]
    fn test_pad_field() {
        assert_eq!(pad_field("W1AW", 10), "W1AW      ");
        assert_eq!(pad_field("W1AW", 4), "W1AW");
        assert_eq!(pad_field("W1AWLONGER", 4), "W1AW");
        assert_eq!(pad_field("", 5), "     ");
    }

    #[test]
    fn test_contact_to_cabrillo() {
        let mut c = Contact::new("W1AW".to_string());
        c.frequency = 14.175;
        c.mode = "CW".to_string();
        c.rst_sent = "599".to_string();
        c.rst_recv = "589".to_string();
        c.qso_date = "2024-01-15".to_string();
        c.qso_time = "120000".to_string();

        let line = contact_to_cabrillo(&c);
        assert!(line.starts_with("QSO: 14175 "));
        assert!(line.contains("W1AW"));
        assert!(line.contains("20240115"));
    }

    #[test]
    fn test_contact_to_cabrillo_ssb() {
        let mut c = Contact::new("K1ABC".to_string());
        c.frequency = 14.250;
        c.mode = "SSB".to_string();
        c.rst_sent = "59".to_string();
        c.rst_recv = "59".to_string();
        c.qso_date = "2024-06-20".to_string();
        c.qso_time = "143000".to_string();

        let line = contact_to_cabrillo(&c);
        assert!(line.contains(" 14250 "));
        assert!(line.contains("K1ABC"));
    }

    #[test]
    fn test_cabrillo_config_default() {
        let config = CabrilloConfig::default();
        assert_eq!(config.contest, "CQ-WW-SSB");
        assert_eq!(config.category_operator, "SINGLE-OP");
        assert_eq!(config.category_transmitter, "ONE");
        assert_eq!(config.category_power, "HIGH");
        assert_eq!(config.category_station, "FIXED");
    }

    #[test]
    fn test_generate_default_filename_format() {
        let filename = generate_default_filename();
        assert!(filename.starts_with("qso_export_"));
        assert!(filename.ends_with(".adi"));
        assert!(filename.len() > 15);
    }

    #[test]
    fn test_generate_cabrillo_filename_format() {
        let filename = generate_cabrillo_filename();
        assert!(filename.starts_with("qso_export_"));
        assert!(filename.ends_with(".log"));
        assert!(filename.len() > 15);
    }
}
