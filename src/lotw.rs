use crate::models::{get_mode_adif_fields, Contact, StationProfile};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

const LOTW_QUERY_URL: &str = "https://lotw.arrl.org/lotwuser/lotwreport.adi";
const LOTW_USER_AGENT: &str = "QSOLink/0.1";

#[derive(Debug, Clone)]
pub struct LotwQslRecord {
    pub call: String,
    pub band: String,
    pub mode: String,
    pub qso_date: String,
    pub time_on: String,
    pub qsl_rcvd: String,
    pub qsldate: Option<String>,
    pub app_lotw_2xqsl: Option<String>,
    pub app_lotw_modegroup: Option<String>,
    pub station_callsign: Option<String>,
    pub freq: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct LotwQueryResult {
    pub records: Vec<LotwQslRecord>,
    pub last_qsl_timestamp: Option<String>,
    pub last_qsorx_timestamp: Option<String>,
    pub num_records: usize,
    pub is_html_error: bool,
}

pub struct LotwClient {
    username: String,
    password: String,
    client: reqwest::blocking::Client,
}

impl LotwClient {
    pub fn new(username: String, password: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(LOTW_USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");
        Self {
            username,
            password,
            client,
        }
    }

    pub fn fetch_confirmations(&self, since: Option<&str>) -> Result<LotwQueryResult, String> {
        let mut url = format!(
            "{}?login={}&password={}&qso_query=1&qso_qsl=yes&qso_withown=yes",
            LOTW_QUERY_URL,
            urlencoding_simple(&self.username),
            urlencoding_simple(&self.password),
        );

        if let Some(ts) = since {
            url.push_str(&format!("&qso_qslsince={}", ts));
        }

        log::info!("Fetching LoTW confirmations...");
        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let body = response
            .text()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        self.parse_response(&body)
    }

    pub fn fetch_accepted(&self, since: Option<&str>) -> Result<LotwQueryResult, String> {
        let mut url = format!(
            "{}?login={}&password={}&qso_query=1&qso_qsl=no&qso_withown=yes",
            LOTW_QUERY_URL,
            urlencoding_simple(&self.username),
            urlencoding_simple(&self.password),
        );

        if let Some(ts) = since {
            url.push_str(&format!("&qso_qsorxsince={}", ts));
        }

        log::info!("Fetching LoTW accepted QSOs...");
        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let body = response
            .text()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        self.parse_response(&body)
    }

    fn parse_response(&self, body: &str) -> Result<LotwQueryResult, String> {
        if !body.contains("<EOH>") && !body.contains("<eoh>") {
            if body.to_lowercase().contains("<html") || body.to_lowercase().contains("<!doctype") {
                return Ok(LotwQueryResult {
                    records: Vec::new(),
                    last_qsl_timestamp: None,
                    last_qsorx_timestamp: None,
                    num_records: 0,
                    is_html_error: true,
                });
            }
        }

        let (header_lines, record_lines) = split_adif(body);

        let mut last_qsl_timestamp: Option<String> = None;
        let mut last_qsorx_timestamp: Option<String> = None;
        let mut num_records: usize = 0;

        for line in &header_lines {
            let upper = line.to_uppercase();
            if upper.starts_with("APP_LOTW_LASTQSL:") {
                last_qsl_timestamp = extract_field_value(line);
            } else if upper.starts_with("APP_LOTW_LASTQSORX:") {
                last_qsorx_timestamp = extract_field_value(line);
            } else if upper.starts_with("APP_LOTW_NUMREC:") {
                if let Some(v) = extract_field_value(line) {
                    num_records = v.parse().unwrap_or(0);
                }
            }
        }

        let mut records = Vec::new();
        for record_text in record_lines {
            if let Some(record) = parse_adif_record(record_text) {
                records.push(record);
            }
        }

        log::info!(
            "LoTW response: {} records, last_qsl={:?}, last_qsorx={:?}",
            records.len(),
            last_qsl_timestamp,
            last_qsorx_timestamp
        );

        Ok(LotwQueryResult {
            records,
            last_qsl_timestamp,
            last_qsorx_timestamp,
            num_records,
            is_html_error: false,
        })
    }

    pub fn test_connection(&self) -> Result<bool, String> {
        match self.fetch_confirmations(None) {
            Ok(result) => {
                if result.is_html_error {
                    Err("LoTW returned an HTML error page. Check your credentials.".to_string())
                } else {
                    Ok(true)
                }
            }
            Err(e) => Err(format!("LoTW connection failed: {}", e)),
        }
    }
}

pub fn export_for_lotw(
    contacts: &[Contact],
    profile: &StationProfile,
    path: &Path,
) -> std::io::Result<usize> {
    let mut file = File::create(path)?;

    writeln!(file, "<ADIF_VERS:5>3.0.4")?;
    writeln!(file, "<PROGRAMID:7>QSOLink")?;
    writeln!(
        file,
        "<PROGRAMVERSION:5>{}.{}.{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH")
    )?;
    if !profile.callsign.is_empty() {
        write_field(&mut file, "STATION_CALLSIGN", &profile.callsign)?;
    }
    if !profile.grid_square.is_empty() {
        write_field(&mut file, "MY_GRIDSQUARE", &profile.grid_square)?;
    }
    if profile.cq_zone > 0 {
        write_field(&mut file, "MY_CQ_ZONE", &profile.cq_zone.to_string())?;
    }
    if profile.itu_zone > 0 {
        write_field(&mut file, "MY_ITU_ZONE", &profile.itu_zone.to_string())?;
    }
    writeln!(file, "<EOH>")?;
    writeln!(file)?;

    let mut count = 0;
    for contact in contacts {
        if contact.call_sign.trim().is_empty() {
            continue;
        }

        write_field(&mut file, "CALL", &contact.call_sign)?;

        let date_clean = contact.qso_date.replace("-", "");
        write_adif_field(&mut file, "QSO_DATE", &date_clean)?;

        let time_clean = pad_time_to_6(&contact.qso_time);
        write_adif_field(&mut file, "TIME_ON", &time_clean)?;

        write_field(&mut file, "BAND", &contact.band)?;

        let (adif_mode, adif_submode) = get_mode_adif_fields(&contact.mode);
        write_field(&mut file, "MODE", &adif_mode)?;
        if let Some(submode) = adif_submode {
            write_field(&mut file, "SUBMODE", &submode)?;
        }

        if contact.frequency > 0.0 {
            let freq_str = format!("{:.5}", contact.frequency);
            write_adif_field(&mut file, "FREQ", &freq_str)?;
        }

        if !contact.rst_sent.is_empty() {
            write_field(&mut file, "RST_SENT", &contact.rst_sent)?;
        }
        if !contact.rst_recv.is_empty() {
            write_field(&mut file, "RST_RCVD", &contact.rst_recv)?;
        }

        if !contact.grid_square.trim().is_empty() {
            write_field(&mut file, "GRIDSQUARE", &contact.grid_square)?;
        }
        if contact.cq_zone > 0 {
            write_field(&mut file, "CQZ", &contact.cq_zone.to_string())?;
        }
        if contact.itu_zone > 0 {
            write_field(&mut file, "ITUZ", &contact.itu_zone.to_string())?;
        }
        if !contact.city.trim().is_empty() {
            write_field(&mut file, "CITY", &contact.city)?;
        }
        if !contact.state.trim().is_empty() {
            write_field(&mut file, "STATE", &contact.state)?;
        }

        writeln!(file, "<EOR>")?;
        count += 1;
    }

    log::info!("Exported {} contacts for LoTW to {:?}", count, path);
    Ok(count)
}

fn pad_time_to_6(time: &str) -> String {
    let cleaned: String = time.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.len() >= 6 {
        cleaned[..6].to_string()
    } else if cleaned.len() == 4 {
        format!("{}00", cleaned)
    } else if cleaned.len() == 2 {
        format!("{}0000", cleaned)
    } else {
        format!("{:0<6}", cleaned)
    }
}

fn write_field<W: Write>(file: &mut W, field_name: &str, value: &str) -> std::io::Result<()> {
    let clean = value.trim();
    write_adif_field(file, field_name, clean)
}

fn write_adif_field<W: Write>(file: &mut W, field_name: &str, value: &str) -> std::io::Result<()> {
    let bytes = value.as_bytes();
    writeln!(file, "<{}:{}>{}", field_name, bytes.len(), value)
}

fn urlencoding_simple(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            _ => {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}

fn split_adif(content: &str) -> (Vec<String>, Vec<&str>) {
    let eoh_pos = content
        .to_uppercase()
        .find("<EOH>")
        .or_else(|| content.to_uppercase().find("<eoh>"));

    match eoh_pos {
        Some(pos) => {
            let header = &content[..pos];
            let body = &content[pos..];
            let eor_positions: Vec<usize> = body
                .to_uppercase()
                .match_indices("<EOR>")
                .map(|(i, _)| i)
                .collect();
            let mut records = Vec::new();
            let mut start = body
                .find("<EOH>")
                .unwrap_or(body.find("<eoh>").unwrap_or(0))
                + 5;
            if body.to_uppercase().starts_with("<EOH>") {
                start = body.find("<EOH>").unwrap_or(0) + 5;
            } else if body.to_uppercase().starts_with("<eoh>") {
                start = body.find("<eoh>").unwrap_or(0) + 5;
            }
            for eor_pos in eor_positions {
                if eor_pos > start {
                    records.push(&body[start..eor_pos]);
                    start = eor_pos + 5;
                }
            }
            (parse_header_lines(header), records)
        }
        None => (Vec::new(), parse_loose_records(content)),
    }
}

fn parse_header_lines(header: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for line in header.lines() {
        let upper = line.to_uppercase();
        if upper.starts_with('<') && !current.is_empty() {
            lines.push(current.trim().to_string());
            current = String::new();
        }
        current.push_str(line);
        current.push('\n');
    }
    if !current.trim().is_empty() {
        lines.push(current.trim().to_string());
    }
    lines
}

fn parse_loose_records(content: &str) -> Vec<&str> {
    let upper = content.to_uppercase();
    let mut records = Vec::new();
    let mut search_start = 0;
    while let Some(eor_pos) = upper[search_start..].find("<EOR>") {
        let actual_pos = search_start + eor_pos;
        if actual_pos > 0 {
            let record_start = find_record_start(content, actual_pos);
            records.push(&content[record_start..actual_pos]);
        }
        search_start = actual_pos + 5;
    }
    records
}

fn find_record_start(content: &str, eor_pos: usize) -> usize {
    let before = &content[..eor_pos];
    let eoh_pos = before
        .to_uppercase()
        .rfind("<EOH>")
        .or_else(|| before.to_uppercase().rfind("<eoh>"));
    match eoh_pos {
        Some(pos) => pos + 5,
        None => 0,
    }
}

fn parse_adif_record(record_text: &str) -> Option<LotwQslRecord> {
    let mut call = String::new();
    let mut band = String::new();
    let mut mode = String::new();
    let mut qso_date = String::new();
    let mut time_on = String::new();
    let mut qsl_rcvd = String::new();
    let mut qsldate: Option<String> = None;
    let mut app_lotw_2xqsl: Option<String> = None;
    let mut app_lotw_modegroup: Option<String> = None;
    let mut station_callsign: Option<String> = None;
    let mut freq: Option<f64> = None;

    let mut pos = 0;
    let text = record_text;

    while pos < text.len() {
        if text[pos..].starts_with('<') {
            if let Some(end_pos) = text[pos..].find('>') {
                let field_part = &text[pos + 1..pos + end_pos];
                pos += end_pos + 1;

                if let Some(colon_pos) = field_part.find(':') {
                    let field_name = field_part[..colon_pos].to_uppercase();
                    let rest = &field_part[colon_pos + 1..];

                    if let Some(len_pos) = rest.find('<') {
                        let len_str = &rest[..len_pos];
                        if let Ok(len) = len_str.parse::<usize>() {
                            let value = &text[pos..pos + len];
                            pos += len;

                            match field_name.as_str() {
                                "CALL" => call = value.to_string(),
                                "BAND" => band = value.to_string(),
                                "MODE" => mode = value.to_string(),
                                "QSO_DATE" => qso_date = value.to_string(),
                                "TIME_ON" => time_on = value.to_string(),
                                "QSL_RCVD" => qsl_rcvd = value.to_string(),
                                "QSLRDATE" => qsldate = Some(value.to_string()),
                                "APP_LOTW_2XQSL" => app_lotw_2xqsl = Some(value.to_string()),
                                "APP_LOTW_MODEGROUP" => {
                                    app_lotw_modegroup = Some(value.to_string())
                                }
                                "STATION_CALLSIGN" => station_callsign = Some(value.to_string()),
                                "FREQ" => freq = value.parse().ok(),
                                _ => {}
                            }
                        }
                    }
                }
            }
        } else {
            pos += 1;
        }
    }

    if call.is_empty() {
        return None;
    }

    Some(LotwQslRecord {
        call,
        band,
        mode,
        qso_date,
        time_on,
        qsl_rcvd,
        qsldate,
        app_lotw_2xqsl,
        app_lotw_modegroup,
        station_callsign,
        freq,
    })
}

fn extract_field_value(line: &str) -> Option<String> {
    if let Some(colon_pos) = line.find(':') {
        let rest = &line[colon_pos + 1..];
        if let Some(gt_pos) = rest.find('>') {
            return Some(rest[..gt_pos].to_string());
        }
    }
    None
}

pub fn match_lotw_record_to_contact(record: &LotwQslRecord, contacts: &[Contact]) -> Option<i64> {
    let date_str = normalize_date(&record.qso_date);
    let time_str = normalize_time(&record.time_on);

    let candidates: Vec<&Contact> = contacts
        .iter()
        .filter(|c| {
            c.call_sign.to_uppercase() == record.call.to_uppercase()
                && normalize_date_for_cmp(&c.qso_date) == date_str
                && c.band.to_uppercase() == record.band.to_uppercase()
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    if candidates.len() == 1 {
        return candidates[0].id;
    }

    if let Some(first_match) = candidates
        .iter()
        .find(|c| normalize_time_for_cmp(&c.qso_time) == time_str)
    {
        return first_match.id;
    }

    if !record.mode.is_empty() {
        if let Some(mode_match) = candidates
            .iter()
            .find(|c| c.mode.to_uppercase() == record.mode.to_uppercase())
        {
            return mode_match.id;
        }
    }

    candidates.first().and_then(|c| c.id)
}

fn normalize_date(date: &str) -> String {
    let cleaned: String = date.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.len() >= 8 {
        format!("{}-{}-{}", &cleaned[0..4], &cleaned[4..6], &cleaned[6..8])
    } else {
        date.to_string()
    }
}

fn normalize_date_for_cmp(date: &str) -> String {
    date.replace("-", "")
}

fn normalize_time(time: &str) -> String {
    let cleaned: String = time.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.len() >= 6 {
        format!("{}:{}:{}", &cleaned[0..2], &cleaned[2..4], &cleaned[4..6])
    } else if cleaned.len() >= 4 {
        format!("{}:{}:00", &cleaned[0..2], &cleaned[2..4])
    } else {
        time.to_string()
    }
}

fn normalize_time_for_cmp(time: &str) -> String {
    let cleaned: String = time.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.len() >= 4 {
        format!("{}:{}", &cleaned[0..2], &cleaned[2..4])
    } else {
        time.to_string()
    }
}

pub fn generate_lotw_filename() -> String {
    let now = chrono::Local::now();
    format!("qsolink_lotw_{}.adi", now.format("%Y%m%d_%H%M"))
}
