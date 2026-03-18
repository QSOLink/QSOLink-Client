use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct QrzSession {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Count")]
    pub count: Option<String>,
    #[serde(rename = "Remark")]
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QrzCallsign {
    #[serde(rename = "call")]
    pub call: String,
    #[serde(rename = "fname")]
    pub fname: Option<String>,
    #[serde(rename = "name")]
    pub name: Option<String>,
    #[serde(rename = "addr1")]
    pub addr1: Option<String>,
    #[serde(rename = "addr2")]
    pub addr2: Option<String>,
    #[serde(rename = "state")]
    pub state: Option<String>,
    #[serde(rename = "zip")]
    pub zip: Option<String>,
    #[serde(rename = "country")]
    pub country: Option<String>,
    #[serde(rename = "grid")]
    pub grid: Option<String>,
    #[serde(rename = "dxcc")]
    pub dxcc: Option<String>,
    #[serde(rename = "lotw")]
    pub lotw: Option<String>,
    #[serde(rename = "eqsl")]
    pub eqsl: Option<String>,
    #[serde(rename = "qsl")]
    pub qsl: Option<String>,
    #[serde(rename = "error")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QrzResponse {
    #[serde(rename = "Session")]
    pub session: Option<QrzSession>,
    #[serde(rename = "Callsign")]
    pub callsign: Option<QrzCallsign>,
}

pub struct QrzClient {
    username: String,
    password: String,
    session_key: Option<String>,
    client: reqwest::blocking::Client,
}

impl QrzClient {
    pub fn new(username: String, password: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            username,
            password,
            session_key: None,
            client,
        }
    }

    pub fn login(&mut self) -> Result<String, String> {
        let url = "https://xml.qrz.com/xml";
        let params = [
            ("username", self.username.as_str()),
            ("password", self.password.as_str()),
            ("agent", "qsolog/0.1"),
        ];

        let response = self
            .client
            .post(url)
            .form(&params)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let text = response.text().map_err(|e| format!("Read failed: {}", e))?;

        let qrz: QrzResponse =
            serde_xml_rs::from_str(&text).map_err(|e| format!("Parse failed: {}", e))?;

        if let Some(session) = qrz.session {
            self.session_key = Some(session.key.clone());
            Ok(session.key)
        } else {
            Err("Failed to get session key".to_string())
        }
    }

    pub fn lookup(&mut self, callsign: &str) -> Result<Option<QrzCallsign>, String> {
        if self.session_key.is_none() {
            self.login()?;
        }

        let url = "https://xml.qrz.com/xml";
        let s = self.session_key.as_ref().unwrap();
        let params = [("s", s.as_str()), ("callsign", callsign)];

        let response = self
            .client
            .post(url)
            .form(&params)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let text = response.text().map_err(|e| format!("Read failed: {}", e))?;

        let qrz: QrzResponse =
            serde_xml_rs::from_str(&text).map_err(|e| format!("Parse failed: {}", e))?;

        if let Some(cs) = qrz.callsign {
            if cs.error.is_some() {
                return Ok(None);
            }
            Ok(Some(cs))
        } else {
            Ok(None)
        }
    }
}

impl Default for QrzClient {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}
