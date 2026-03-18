use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigConfig {
    pub host: String,
    pub port: u16,
    pub poll_interval_ms: u32,
    pub auto_reconnect: bool,
}

impl Default for RigConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 4532,
            poll_interval_ms: 1000,
            auto_reconnect: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigState {
    pub frequency: f64,
    pub mode: String,
    pub vfo: String,
    pub ptt: bool,
    pub connected: bool,
    pub error_message: Option<String>,
}

impl Default for RigState {
    fn default() -> Self {
        Self {
            frequency: 0.0,
            mode: "".to_string(),
            vfo: "".to_string(),
            ptt: false,
            connected: false,
            error_message: None,
        }
    }
}

pub struct RigCtlClient {
    config: RigConfig,
    state: RigState,
    stream: Mutex<Option<BufReader<TcpStream>>>,
}

impl Clone for RigCtlClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: self.state.clone(),
            stream: Mutex::new(None),
        }
    }
}

impl RigCtlClient {
    pub fn new(config: RigConfig) -> Self {
        Self {
            config,
            state: RigState::default(),
            stream: Mutex::new(None),
        }
    }

    pub fn config(&self) -> &RigConfig {
        &self.config
    }

    pub fn state(&self) -> &RigState {
        &self.state
    }

    pub async fn connect(&mut self) -> Result<(), String> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?;

        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        let reader = BufReader::new(stream);
        *self.stream.lock().await = Some(reader);
        
        self.state.connected = true;
        self.state.error_message = None;
        
        log::info!("Connected to rigctld at {}:{}", self.config.host, self.config.port);
        
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), String> {
        let mut stream_guard = self.stream.lock().await;
        if let Some(reader) = stream_guard.take() {
            let mut writer = reader.into_inner();
            let _ = writer.shutdown().await;
        }
        self.state.connected = false;
        self.state.error_message = None;
        
        log::info!("Disconnected from rigctld");
        
        Ok(())
    }

    pub async fn send_command(&mut self, cmd: &str) -> Result<String, String> {
        let mut stream_guard = self.stream.lock().await;
        let reader = stream_guard.as_mut().ok_or("Not connected")?;
        let writer = reader.get_mut();

        writer
            .write_all(cmd.as_bytes())
            .await
            .map_err(|e| format!("Write failed: {}", e))?;
        writer
            .flush()
            .await
            .map_err(|e| format!("Flush failed: {}", e))?;

        let mut response = String::new();
        reader
            .read_line(&mut response)
            .await
            .map_err(|e| format!("Read failed: {}", e))?;

        Ok(response.trim().to_string())
    }

    pub async fn get_frequency(&mut self) -> Result<f64, String> {
        let response = self.send_command("f\n").await?;
        
        response.parse::<f64>().map_err(|e| format!("Parse error: {} - raw: {}", e, response))
    }

    pub async fn get_mode(&mut self) -> Result<(String, String), String> {
        let response = self.send_command("m\n").await?;
        
        let parts: Vec<&str> = response.split(',').collect();
        if parts.len() >= 2 {
            Ok((parts[0].to_string(), parts[1].to_string()))
        } else {
            Err(format!("Invalid mode response: {}", response))
        }
    }

    pub async fn set_frequency(&mut self, freq: f64) -> Result<(), String> {
        let cmd = format!("F {}\n", freq);
        let response = self.send_command(&cmd).await?;
        
        if response == "RPRT 0" {
            Ok(())
        } else {
            Err(format!("Set frequency failed: {}", response))
        }
    }

    pub async fn set_mode(&mut self, mode: &str, bandwidth: &str) -> Result<(), String> {
        let cmd = format!("M {} {}\n", mode, bandwidth);
        let response = self.send_command(&cmd).await?;
        
        if response == "RPRT 0" {
            Ok(())
        } else {
            Err(format!("Set mode failed: {}", response))
        }
    }

    pub async fn get_vfo(&mut self) -> Result<String, String> {
        let response = self.send_command("v\n").await?;
        Ok(response)
    }

    pub async fn ptt(&mut self, on: bool) -> Result<(), String> {
        let cmd = format!("T {}\n", if on { 1 } else { 0 });
        let response = self.send_command(&cmd).await?;
        
        if response == "RPRT 0" {
            self.state.ptt = on;
            Ok(())
        } else {
            Err(format!("PTT failed: {}", response))
        }
    }

    pub async fn update_state(&mut self) {
        if !self.state.connected {
            return;
        }

        match self.get_frequency().await {
            Ok(freq) => self.state.frequency = freq,
            Err(e) => {
                self.state.error_message = Some(e);
                return;
            }
        }

        match self.get_mode().await {
            Ok((mode, _)) => self.state.mode = mode,
            Err(e) => {
                self.state.error_message = Some(e);
            }
        }
    }

    pub fn set_config(&mut self, config: RigConfig) {
        self.config = config;
    }

    pub fn get_config(&self) -> RigConfig {
        self.config.clone()
    }

    pub fn format_frequency(&self) -> String {
        if self.state.frequency > 0.0 {
            let freq = self.state.frequency;
            if freq >= 1_000_000.0 {
                format!("{:.3} MHz", freq / 1_000_000.0)
            } else if freq >= 1_000.0 {
                format!("{:.3} kHz", freq / 1_000.0)
            } else {
                format!("{:.0} Hz", freq)
            }
        } else {
            "-- MHz".to_string()
        }
    }
}

pub fn mode_to_string(mode_code: &str) -> &'static str {
    match mode_code {
        "0" => "USB",
        "1" => "LSB",
        "2" => "CW",
        "3" => "CWR",
        "4" => "AM",
        "5" => "FM",
        "6" => "WFM",
        "7" => "FT8",
        "8" => "FT4",
        "9" => "RTTY",
        "10" => "RTTYR",
        _ => "Unknown",
    }
}
