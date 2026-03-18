use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Option<i64>,
    pub call_sign: String,
    pub name: String,
    pub qth: String,
    pub frequency: f64,
    pub band: String,
    pub mode: String,
    pub rst_sent: String,
    pub rst_recv: String,
    pub notes: String,
    pub qso_date: String,
    pub qso_time: String,
    pub created_at: Option<String>,
}

#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl Contact {
    pub fn new(call_sign: String) -> Self {
        let now = Local::now();
        Self {
            id: None,
            call_sign,
            name: String::new(),
            qth: String::new(),
            frequency: 14.175,
            band: "20m".to_string(),
            mode: "SSB".to_string(),
            rst_sent: "59".to_string(),
            rst_recv: "59".to_string(),
            notes: String::new(),
            qso_date: now.format("%Y-%m-%d").to_string(),
            qso_time: now.format("%H%M").to_string(),
            created_at: None,
        }
    }

    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        match validate_callsign(&self.call_sign) {
            Err(e) => errors.push(e),
            _ => {}
        }

        if !self.name.is_empty() {
            if self.name.len() > 50 {
                errors.push(ValidationError {
                    field: "name".to_string(),
                    message: "Name must be 50 characters or less".to_string(),
                });
            }
        }

        if !self.qth.is_empty() {
            if self.qth.len() > 100 {
                errors.push(ValidationError {
                    field: "qth".to_string(),
                    message: "QTH must be 100 characters or less".to_string(),
                });
            }
        }

        if self.frequency < 0.0 || self.frequency > 25000.0 {
            errors.push(ValidationError {
                field: "frequency".to_string(),
                message: "Frequency must be between 0 and 25000 MHz".to_string(),
            });
        }

        if !self.notes.is_empty() && self.notes.len() > 1000 {
            errors.push(ValidationError {
                field: "notes".to_string(),
                message: "Notes must be 1000 characters or less".to_string(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn validate_callsign(callsign: &str) -> Result<(), ValidationError> {
    if callsign.is_empty() {
        return Err(ValidationError {
            field: "call_sign".to_string(),
            message: "Call sign is required".to_string(),
        });
    }

    let cleaned = callsign.trim().to_uppercase();

    if cleaned.len() < 3 {
        return Err(ValidationError {
            field: "call_sign".to_string(),
            message: "Call sign must be at least 3 characters".to_string(),
        });
    }

    if cleaned.len() > 7 {
        return Err(ValidationError {
            field: "call_sign".to_string(),
            message: "Call sign must be 7 characters or less".to_string(),
        });
    }

    let valid_chars = cleaned
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '/' || c == '-');

    if !valid_chars {
        return Err(ValidationError {
            field: "call_sign".to_string(),
            message: "Call sign can only contain letters, numbers, / and -".to_string(),
        });
    }

    Ok(())
}

pub fn sanitize_string(input: &str, max_len: usize) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
        .take(max_len)
        .collect()
}

pub const BANDS: &[&str] = &[
    "160m", "80m", "60m", "40m", "30m", "20m", "17m", "15m", "12m", "10m", "6m", "2m", "70cm",
    "23cm",
];

pub const MODES: &[&str] = &[
    "SSB", "CW", "FM", "AM", "DIG", "DATA", "RTTY", "PSK31", "PSK63", "FT8", "FT4", "JS8",
];

pub fn frequency_to_band(freq_mhz: f64) -> Option<&'static str> {
    match freq_mhz {
        f if f >= 1.8 && f < 2.0 => Some("160m"),
        f if f >= 3.5 && f < 4.0 => Some("80m"),
        f if f >= 5.3 && f < 5.4 => Some("60m"),
        f if f >= 7.0 && f < 7.3 => Some("40m"),
        f if f >= 10.1 && f < 10.15 => Some("30m"),
        f if f >= 14.0 && f < 14.35 => Some("20m"),
        f if f >= 18.068 && f < 18.168 => Some("17m"),
        f if f >= 21.0 && f < 21.45 => Some("15m"),
        f if f >= 24.89 && f < 24.99 => Some("12m"),
        f if f >= 28.0 && f < 29.7 => Some("10m"),
        f if f >= 50.0 && f < 54.0 => Some("6m"),
        f if f >= 144.0 && f < 148.0 => Some("2m"),
        f if f >= 420.0 && f < 450.0 => Some("70cm"),
        f if f >= 1240.0 && f < 1300.0 => Some("23cm"),
        _ => None,
    }
}

pub fn band_to_frequency(band: &str) -> Option<f64> {
    match band {
        "160m" => Some(1.9),
        "80m" => Some(3.75),
        "60m" => Some(5.35),
        "40m" => Some(7.15),
        "30m" => Some(10.125),
        "20m" => Some(14.175),
        "17m" => Some(18.118),
        "15m" => Some(21.225),
        "12m" => Some(24.94),
        "10m" => Some(28.85),
        "6m" => Some(52.0),
        "2m" => Some(146.0),
        "70cm" => Some(435.0),
        "23cm" => Some(1270.0),
        _ => None,
    }
}
