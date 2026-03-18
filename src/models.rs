use chrono::{Local, Utc};
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
    pub submode: String,
    pub rst_sent: String,
    pub rst_recv: String,
    pub notes: String,
    pub qso_date: String,
    pub qso_time: String,
    pub created_at: Option<String>,
    pub qso_count: i32,
    pub city: String,
    pub state: String,
    pub county: String,
    pub grid_square: String,
    pub cq_zone: i32,
    pub itu_zone: i32,
    pub lotw_submitted: bool,
    pub lotw_confirmed: bool,
    pub lotw_submitted_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationProfile {
    pub callsign: String,
    pub grid_square: String,
    pub dxcc_entity: i32,
    pub cq_zone: i32,
    pub itu_zone: i32,
    pub arl_section: String,
}

impl StationProfile {
    pub fn new() -> Self {
        Self {
            callsign: String::new(),
            grid_square: String::new(),
            dxcc_entity: 0,
            cq_zone: 0,
            itu_zone: 0,
            arl_section: String::new(),
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.callsign.trim().is_empty() && !self.grid_square.trim().is_empty()
    }
}

impl Default for StationProfile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LotwStatus {
    NotSubmitted,
    Submitted,
    Confirmed,
}

impl Contact {
    pub fn lotw_status(&self) -> LotwStatus {
        if self.lotw_confirmed {
            LotwStatus::Confirmed
        } else if self.lotw_submitted {
            LotwStatus::Submitted
        } else {
            LotwStatus::NotSubmitted
        }
    }

    pub fn can_submit_to_lotw(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.call_sign.trim().is_empty() {
            missing.push("call_sign");
        }
        if self.qso_date.trim().is_empty() {
            missing.push("qso_date");
        }
        if self.qso_time.trim().is_empty() {
            missing.push("qso_time");
        }
        if self.band.trim().is_empty() {
            missing.push("band");
        }
        if self.mode.trim().is_empty() {
            missing.push("mode");
        }
        missing
    }

    pub fn missing_lotw_station_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.grid_square.trim().is_empty() {
            missing.push("grid_square");
        }
        missing
    }
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

pub fn validate_grid_square(input: &str) -> Result<(), ValidationError> {
    let g = input.trim().to_uppercase();
    if g.is_empty() {
        return Err(ValidationError {
            field: "grid_square".to_string(),
            message: "Grid square is required".to_string(),
        });
    }

    let len = g.len();
    if len != 4 && len != 6 && len != 8 {
        return Err(ValidationError {
            field: "grid_square".to_string(),
            message: "Grid square must be 4, 6, or 8 characters (e.g. EM75, EM75AX, EM75AX01)"
                .to_string(),
        });
    }

    let first_two = &g[..2];
    if !first_two.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(ValidationError {
            field: "grid_square".to_string(),
            message: "First two characters of grid square must be letters (e.g. EM75)".to_string(),
        });
    }

    let remainder = &g[2..];
    if !remainder.chars().all(|c| c.is_ascii_digit()) {
        return Err(ValidationError {
            field: "grid_square".to_string(),
            message: "Last 2-6 characters of grid square must be numbers (e.g. EM75)".to_string(),
        });
    }

    let field1: u32 = remainder[..2].parse().map_err(|_| ValidationError {
        field: "grid_square".to_string(),
        message: "Invalid grid square field 1".to_string(),
    })?;
    if field1 > 89 {
        return Err(ValidationError {
            field: "grid_square".to_string(),
            message: "Invalid grid square — field 1 must be 00-89".to_string(),
        });
    }

    if len >= 4 {
        let field2: u32 = remainder[2..4].parse().map_err(|_| ValidationError {
            field: "grid_square".to_string(),
            message: "Invalid grid square field 2".to_string(),
        })?;
        if field2 > 89 {
            return Err(ValidationError {
                field: "grid_square".to_string(),
                message: "Invalid grid square — field 2 must be 00-89".to_string(),
            });
        }
    }

    if len >= 6 {
        let subfield1: u32 = remainder[4..5].parse().map_err(|_| ValidationError {
            field: "grid_square".to_string(),
            message: "Invalid grid square subfield".to_string(),
        })?;
        if subfield1 > 23 {
            return Err(ValidationError {
                field: "grid_square".to_string(),
                message: "Invalid grid square — subfield 1 must be A-X (0-23)".to_string(),
            });
        }
    }

    if len == 8 {
        let subfield2: u32 = remainder[5..6].parse().map_err(|_| ValidationError {
            field: "grid_square".to_string(),
            message: "Invalid grid square subfield".to_string(),
        })?;
        if subfield2 > 23 {
            return Err(ValidationError {
                field: "grid_square".to_string(),
                message: "Invalid grid square — subfield 2 must be A-X (0-23)".to_string(),
            });
        }
    }

    Ok(())
}

impl Contact {
    pub fn new(call_sign: String) -> Self {
        let now_utc = Utc::now();
        let now_local = Local::now();
        Self {
            id: None,
            call_sign,
            name: String::new(),
            qth: String::new(),
            frequency: 14.175,
            band: "20m".to_string(),
            mode: "SSB".to_string(),
            submode: String::new(),
            rst_sent: "59".to_string(),
            rst_recv: "59".to_string(),
            notes: String::new(),
            qso_date: now_utc.format("%Y-%m-%d").to_string(),
            qso_time: now_local.format("%H%M%S").to_string(),
            created_at: None,
            qso_count: 1,
            city: String::new(),
            state: String::new(),
            county: String::new(),
            grid_square: String::new(),
            cq_zone: 0,
            itu_zone: 0,
            lotw_submitted: false,
            lotw_confirmed: false,
            lotw_submitted_date: None,
        }
    }

    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        match validate_callsign(&self.call_sign) {
            Err(e) => errors.push(e),
            _ => {}
        }

        if let Err(e) = validate_qso_time(&self.qso_time) {
            errors.push(e);
        }

        if let Err(e) = validate_qso_date(&self.qso_date) {
            errors.push(e);
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

pub fn validate_qso_time(time: &str) -> Result<(), ValidationError> {
    let t = time.trim();
    if t.is_empty() {
        return Err(ValidationError {
            field: "qso_time".to_string(),
            message: "QSO time is required (HHMMSS, UTC)".to_string(),
        });
    }
    if t.len() != 6 || !t.chars().all(|c| c.is_ascii_digit()) {
        return Err(ValidationError {
            field: "qso_time".to_string(),
            message: "QSO time must be 6 digits (HHMMSS, UTC)".to_string(),
        });
    }
    let hours: u32 = t[0..2].parse().unwrap_or(99);
    let mins: u32 = t[2..4].parse().unwrap_or(99);
    let secs: u32 = t[4..6].parse().unwrap_or(99);
    if hours > 23 || mins > 59 || secs > 59 {
        return Err(ValidationError {
            field: "qso_time".to_string(),
            message: "QSO time has invalid hours, minutes, or seconds".to_string(),
        });
    }
    Ok(())
}

pub fn validate_qso_date(date: &str) -> Result<(), ValidationError> {
    let d = date.trim();
    if d.is_empty() {
        return Err(ValidationError {
            field: "qso_date".to_string(),
            message: "QSO date is required (YYYY-MM-DD)".to_string(),
        });
    }
    if d.len() != 10 || d.chars().nth(4) != Some('-') || d.chars().nth(7) != Some('-') {
        return Err(ValidationError {
            field: "qso_date".to_string(),
            message: "QSO date must be in YYYY-MM-DD format".to_string(),
        });
    }
    let year: u32 = d[0..4].parse().unwrap_or(0);
    let month: u32 = d[5..7].parse().unwrap_or(0);
    let day: u32 = d[8..10].parse().unwrap_or(0);
    if year < 1900 || year > 2100 || month < 1 || month > 12 || day < 1 || day > 31 {
        return Err(ValidationError {
            field: "qso_date".to_string(),
            message: "QSO date has invalid year, month, or day".to_string(),
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

pub struct ModeInfo {
    pub value: &'static str,
    pub label: &'static str,
    pub is_adif_enumerated: bool,
    pub is_digital_submode: bool,
    pub adif_mode: &'static str,
    pub adif_submode: Option<&'static str>,
}

pub const MODE_OPTIONS: &[ModeInfo] = &[
    ModeInfo {
        value: "CW",
        label: "CW",
        is_adif_enumerated: true,
        is_digital_submode: false,
        adif_mode: "CW",
        adif_submode: None,
    },
    ModeInfo {
        value: "SSB",
        label: "SSB",
        is_adif_enumerated: true,
        is_digital_submode: false,
        adif_mode: "SSB",
        adif_submode: None,
    },
    ModeInfo {
        value: "USB",
        label: "USB",
        is_adif_enumerated: true,
        is_digital_submode: false,
        adif_mode: "USB",
        adif_submode: None,
    },
    ModeInfo {
        value: "LSB",
        label: "LSB",
        is_adif_enumerated: true,
        is_digital_submode: false,
        adif_mode: "LSB",
        adif_submode: None,
    },
    ModeInfo {
        value: "FM",
        label: "FM",
        is_adif_enumerated: true,
        is_digital_submode: false,
        adif_mode: "FM",
        adif_submode: None,
    },
    ModeInfo {
        value: "AM",
        label: "AM",
        is_adif_enumerated: true,
        is_digital_submode: false,
        adif_mode: "AM",
        adif_submode: None,
    },
    ModeInfo {
        value: "RTTY",
        label: "RTTY",
        is_adif_enumerated: true,
        is_digital_submode: false,
        adif_mode: "RTTY",
        adif_submode: None,
    },
    ModeInfo {
        value: "DATA",
        label: "DATA",
        is_adif_enumerated: true,
        is_digital_submode: false,
        adif_mode: "DATA",
        adif_submode: None,
    },
    ModeInfo {
        value: "FT8",
        label: "FT8",
        is_adif_enumerated: false,
        is_digital_submode: true,
        adif_mode: "DATA",
        adif_submode: Some("FT8"),
    },
    ModeInfo {
        value: "FT4",
        label: "FT4",
        is_adif_enumerated: false,
        is_digital_submode: true,
        adif_mode: "DATA",
        adif_submode: Some("FT4"),
    },
    ModeInfo {
        value: "PSK31",
        label: "PSK31",
        is_adif_enumerated: false,
        is_digital_submode: true,
        adif_mode: "DATA",
        adif_submode: Some("PSK31"),
    },
    ModeInfo {
        value: "PSK63",
        label: "PSK63",
        is_adif_enumerated: false,
        is_digital_submode: true,
        adif_mode: "DATA",
        adif_submode: Some("PSK63"),
    },
    ModeInfo {
        value: "JS8",
        label: "JS8",
        is_adif_enumerated: false,
        is_digital_submode: true,
        adif_mode: "DATA",
        adif_submode: Some("JS8"),
    },
    ModeInfo {
        value: "JS4",
        label: "JS4",
        is_adif_enumerated: false,
        is_digital_submode: true,
        adif_mode: "DATA",
        adif_submode: Some("JS4"),
    },
    ModeInfo {
        value: "MFSK",
        label: "MFSK",
        is_adif_enumerated: false,
        is_digital_submode: true,
        adif_mode: "DATA",
        adif_submode: Some("MFSK"),
    },
    ModeInfo {
        value: "OLIVIA",
        label: "OLIVIA",
        is_adif_enumerated: false,
        is_digital_submode: true,
        adif_mode: "DATA",
        adif_submode: Some("OLIVIA"),
    },
    ModeInfo {
        value: "RTTY",
        label: "RTTY",
        is_adif_enumerated: true,
        is_digital_submode: false,
        adif_mode: "RTTY",
        adif_submode: None,
    },
    ModeInfo {
        value: "DIG",
        label: "DIG (Digital)",
        is_adif_enumerated: true,
        is_digital_submode: false,
        adif_mode: "DATA",
        adif_submode: None,
    },
];

pub fn find_mode_info(value: &str) -> Option<&'static ModeInfo> {
    MODE_OPTIONS.iter().find(|m| m.value == value)
}

pub fn get_mode_adif_fields(mode: &str) -> (String, Option<String>) {
    if let Some(info) = find_mode_info(mode) {
        if let Some(submode) = info.adif_submode {
            return (info.adif_mode.to_string(), Some(submode.to_string()));
        }
        return (info.adif_mode.to_string(), None);
    }
    ("DATA".to_string(), Some(mode.to_string()))
}

pub fn mode_needs_warning(mode: &str) -> bool {
    find_mode_info(mode)
        .map(|m| m.is_digital_submode)
        .unwrap_or(false)
}

pub const SUBMODE_MODES: &[&str] = &[
    "FT8", "FT4", "PSK31", "PSK63", "JS8", "JS4", "MFSK", "OLIVIA",
];

pub fn is_submode(mode: &str) -> bool {
    SUBMODE_MODES.contains(&mode)
}

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
