use crate::db::Database;
use crate::lotw::{self, LotwClient};
use crate::models::{
    validate_callsign, validate_grid_square, validate_qso_date, validate_qso_time,
    mode_needs_warning, Contact, LotwStatus, StationProfile, BANDS,
};
use crate::remote_db::{DatabaseConfig, DatabaseType, RemoteDatabase};
use crate::rigctl::{RigCtlClient, RigConfig};
use crate::security::{CredentialStore, LotwSyncState};
use chrono::{Local, Utc};
use eframe::egui;

const SYNC_INTERVAL_SECS: u64 = 3600;

pub struct QsoApp {
    db: Database,
    remote_db: Option<RemoteDatabase>,
    contacts: Vec<Contact>,
    search_query: String,
    new_contact: Contact,
    selected_contact: Option<Contact>,
    status_message: String,
    show_delete_confirm: bool,
    contact_to_delete: Option<i64>,
    qrz_client: Option<crate::qrz::QrzClient>,
    qrz_username: String,
    qrz_password: String,
    qrz_username_backup: String,
    qrz_password_backup: String,
    show_qrz_settings: bool,
    credential_store: CredentialStore,
    show_db_settings: bool,
    db_config: DatabaseConfig,
    use_remote_db: bool,
    connection_test_result: Option<String>,
    show_rig_settings: bool,
    rig_client: Option<RigCtlClient>,
    rig_config: RigConfig,
    rig_status_message: Option<String>,
    show_theme_settings: bool,
    current_theme: String,
    show_callsign_details: Option<Contact>,
    callsign_suggestions: Vec<Contact>,
    show_lotw_settings: bool,
    lotw_username: String,
    lotw_password: String,
    lotw_username_backup: String,
    lotw_password_backup: String,
    lotw_client: Option<LotwClient>,
    station_profile: StationProfile,
    station_profile_backup: StationProfile,
    last_sync_time: Option<String>,
    last_sync_instant: std::time::Instant,
    sync_in_progress: bool,
    sync_error: Option<String>,
    lotw_submit_error: Option<String>,
    lotw_submit_success: Option<String>,
    frame_counter: u64,
    mode_custom_text: String,
    use_custom_mode: bool,
}

impl QsoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, db_path: Option<std::path::PathBuf>) -> Self {
        env_logger::init();

        let db = Database::new(db_path).expect("Failed to initialize database");
        let contacts = db.get_all_contacts().unwrap_or_default();
        let new_contact = Contact::new(String::new());

        let credential_store = CredentialStore::new();
        let (qrz_username, qrz_password) = credential_store
            .load_credentials()
            .unwrap_or((String::new(), String::new()));

        let (lotw_username, lotw_password) = credential_store
            .load_lotw_credentials()
            .unwrap_or((String::new(), String::new()));

        let station_profile = credential_store
            .load_station_profile()
            .unwrap_or_else(StationProfile::new);

        let lotw_client = if !lotw_username.is_empty() && !lotw_password.is_empty() {
            Some(LotwClient::new(lotw_username.clone(), lotw_password.clone()))
        } else {
            None
        };

        let sync_state = credential_store.load_lotw_sync_state();
        let last_sync_time = sync_state.last_sync.clone();

        let needs_lotw_setup = lotw_username.is_empty()
            || lotw_password.is_empty()
            || !station_profile.is_complete();

        log::info!(
            "QSOLink started - QRZ: {}, LoTW: {}, Station: {}",
            !qrz_username.is_empty(),
            !lotw_username.is_empty(),
            station_profile.is_complete()
        );

        let mut app = Self {
            db,
            remote_db: None,
            contacts,
            search_query: String::new(),
            new_contact,
            selected_contact: None,
            status_message: String::from("Ready"),
            show_delete_confirm: false,
            contact_to_delete: None,
            qrz_client: None,
            qrz_username,
            qrz_password,
            qrz_username_backup: String::new(),
            qrz_password_backup: String::new(),
            show_qrz_settings: false,
            credential_store,
            show_db_settings: false,
            db_config: DatabaseConfig::default(),
            use_remote_db: false,
            connection_test_result: None,
            show_rig_settings: false,
            rig_client: None,
            rig_config: RigConfig::default(),
            rig_status_message: None,
            show_theme_settings: false,
            current_theme: "Dark".to_string(),
            show_callsign_details: None,
            callsign_suggestions: Vec::new(),
            show_lotw_settings: false,
            lotw_username,
            lotw_password,
            lotw_username_backup: String::new(),
            lotw_password_backup: String::new(),
            lotw_client,
            station_profile,
            station_profile_backup: StationProfile::new(),
            last_sync_time,
            last_sync_instant: std::time::Instant::now(),
            sync_in_progress: false,
            sync_error: None,
            lotw_submit_error: None,
            lotw_submit_success: None,
            frame_counter: 0,
            mode_custom_text: String::new(),
            use_custom_mode: false,
        };

        if needs_lotw_setup {
            app.station_profile_backup = app.station_profile.clone();
            app.lotw_username_backup = app.lotw_username.clone();
            app.lotw_password_backup = app.lotw_password.clone();
            app.show_lotw_settings = true;
        }

        if app.lotw_client.is_some() {
            app.do_lotw_sync();
        }

        app
    }

    fn add_contact(&mut self) {
        let callsign = self.new_contact.call_sign.trim().to_string();

        if callsign.is_empty() {
            self.status_message = "Error: Call sign is required".to_string();
            return;
        }

        if let Err(e) = validate_callsign(&callsign) {
            self.status_message = format!("Error: {}", e);
            return;
        }

        if let Err(e) = validate_qso_time(&self.new_contact.qso_time) {
            self.status_message = format!("Error: {}", e);
            return;
        }

        if let Err(e) = validate_qso_date(&self.new_contact.qso_date) {
            self.status_message = format!("Error: {}", e);
            return;
        }

        if !self.new_contact.grid_square.trim().is_empty() {
            if let Err(e) = validate_grid_square(&self.new_contact.grid_square) {
                self.status_message = format!("Error: {}", e);
                return;
            }
        }

        self.new_contact.call_sign = callsign;

        match self.new_contact.validate() {
            Ok(_) => {}
            Err(errors) => {
                self.status_message = format!("Validation error: {}", errors.first().unwrap());
                return;
            }
        }

        if self.use_remote_db {
            if let Some(ref remote_db) = self.remote_db {
                let rt = tokio::runtime::Runtime::new().unwrap();
                match rt.block_on(remote_db.insert_contact(&self.new_contact)) {
                    Ok(id) => {
                        self.status_message = format!(
                            "Contact {} added to remote DB (ID: {})",
                            self.new_contact.call_sign, id
                        );
                        self.new_contact = Contact::new(String::new());
                        self.refresh_contacts();
                    }
                    Err(e) => {
                        self.status_message = format!("Error adding contact to remote DB: {}", e);
                    }
                }
                return;
            }
        }

        match self.db.insert_contact(&self.new_contact) {
            Ok(id) => {
                self.status_message =
                    format!("Contact {} added (ID: {})", self.new_contact.call_sign, id);
                self.new_contact = Contact::new(String::new());
                self.refresh_contacts();
            }
            Err(e) => {
                self.status_message = format!("Error adding contact: {}", e);
            }
        }
    }

    fn refresh_contacts(&mut self) {
        log::info!("refresh_contacts called with query: '{}'", self.search_query);
        
        if self.use_remote_db {
            if let Some(ref remote_db) = self.remote_db {
                let rt = tokio::runtime::Runtime::new().unwrap();
                self.contacts = if self.search_query.is_empty() {
                    rt.block_on(remote_db.get_all_contacts())
                        .unwrap_or_default()
                } else {
                    rt.block_on(remote_db.search_contacts(&self.search_query))
                        .unwrap_or_default()
                };
                return;
            }
        }
        
        let mut filtered_contacts: Vec<Contact> = if self.search_query.is_empty() {
            self.db.get_all_contacts().unwrap_or_default()
        } else {
            let results = self.db.search_contacts(&self.search_query).unwrap_or_default();
            log::info!("SQL returned {} results", results.len());
            results
        };
        
        if !self.search_query.is_empty() {
            let query_lower = self.search_query.to_lowercase();
            let before_count = filtered_contacts.len();
            filtered_contacts.retain(|c| c.call_sign.to_lowercase().starts_with(&query_lower));
            log::info!("After Rust filter: {} -> {} results", before_count, filtered_contacts.len());
        }
        
        self.contacts = filtered_contacts;
    }

    fn delete_contact(&mut self, id: i64) {
        if self.use_remote_db {
            if let Some(ref remote_db) = self.remote_db {
                let rt = tokio::runtime::Runtime::new().unwrap();
                match rt.block_on(remote_db.delete_contact(id)) {
                    Ok(_) => {
                        self.status_message = format!("Contact {} deleted from remote DB", id);
                        self.refresh_contacts();
                    }
                    Err(e) => {
                        self.status_message = format!("Error deleting from remote DB: {}", e);
                    }
                }
                return;
            }
        }
        match self.db.delete_contact(id) {
            Ok(_) => {
                self.status_message = format!("Contact {} deleted", id);
                self.refresh_contacts();
            }
            Err(e) => {
                self.status_message = format!("Error deleting contact: {}", e);
            }
        }
    }

    fn export_adif(&mut self) {
        use std::path::PathBuf;

        let default_name = crate::export::generate_default_filename();
        let path = PathBuf::from(&default_name);

        match crate::export::export_adif(&self.contacts, &path) {
            Ok(_) => {
                self.status_message = format!(
                    "Exported {} contacts to {}",
                    self.contacts.len(),
                    default_name
                );
            }
            Err(e) => {
                self.status_message = format!("Export failed: {}", e);
            }
        }
    }

    fn export_cabrillo(&mut self) {
        use std::path::PathBuf;

        let config = crate::export::CabrilloConfig::default();
        let default_name = crate::export::generate_cabrillo_filename();
        let path = PathBuf::from(&default_name);

        match crate::export::export_cabrillo(&self.contacts, &path, &config) {
            Ok(_) => {
                self.status_message = format!(
                    "Exported {} contacts to {}",
                    self.contacts.len(),
                    default_name
                );
            }
            Err(e) => {
                self.status_message = format!("Cabrillo export failed: {}", e);
            }
        }
    }

    fn do_lookup(&mut self) {
        let callsign = self.new_contact.call_sign.clone().trim().to_string();
        log::info!("QRZ lookup called for: {}", callsign);

        if callsign.is_empty() {
            self.status_message = "Enter a callsign first".to_string();
            return;
        }

        if let Err(e) = validate_callsign(&callsign) {
            self.status_message = format!("Invalid callsign: {}", e);
            return;
        }

        if self.qrz_client.is_none() {
            if self.qrz_username.is_empty() || self.qrz_password.is_empty() {
                self.status_message =
                    "Configure QRZ credentials first (click Settings)".to_string();
                log::warn!("QRZ credentials not configured");
                return;
            }
            self.qrz_client = Some(crate::qrz::QrzClient::new(
                self.qrz_username.clone(),
                self.qrz_password.clone(),
            ));
        }

        self.status_message = "Looking up on QRZ...".to_string();

        let client = self.qrz_client.as_mut().unwrap();

        match client.lookup(&callsign) {
            Ok(Some(qrz)) => {
                if let Some(ref fname) = qrz.fname {
                    if !fname.is_empty() {
                        if let Some(ref name) = qrz.name {
                            self.new_contact.name = format!("{} {}", fname, name);
                        } else {
                            self.new_contact.name = fname.clone();
                        }
                    }
                }
                if let Some(ref state) = qrz.state {
                    if let Some(ref country) = qrz.country {
                        self.new_contact.qth = if state.is_empty() {
                            country.clone()
                        } else {
                            format!("{}, {}", state, country)
                        };
                    }
                } else if let Some(ref country) = qrz.country {
                    self.new_contact.qth = country.clone();
                }
                
                if let Some(ref addr2) = qrz.addr2 {
                    self.new_contact.city = addr2.clone();
                }
                if let Some(ref state) = qrz.state {
                    self.new_contact.state = state.clone();
                }
                if let Some(ref grid) = qrz.grid {
                    self.new_contact.grid_square = grid.clone();
                }

                self.status_message = format!("Found: {} - {}", qrz.call, self.new_contact.name);
            }
            Ok(None) => {
                self.status_message = "Callsign not found on QRZ".to_string();
            }
            Err(e) => {
                if e.contains("session") || e.contains("login") {
                    self.qrz_client = Some(crate::qrz::QrzClient::new(
                        self.qrz_username.clone(),
                        self.qrz_password.clone(),
                    ));

                    let client = self.qrz_client.as_mut().unwrap();
                    match client.lookup(&callsign) {
                        Ok(Some(qrz)) => {
                            if let Some(ref fname) = qrz.fname {
                                if !fname.is_empty() {
                                    if let Some(ref name) = qrz.name {
                                        self.new_contact.name = format!("{} {}", fname, name);
                                    } else {
                                        self.new_contact.name = fname.clone();
                                    }
                                }
                            }
                            if let Some(ref state) = qrz.state {
                                if let Some(ref country) = qrz.country {
                                    self.new_contact.qth = if state.is_empty() {
                                        country.clone()
                                    } else {
                                        format!("{}, {}", state, country)
                                    };
                                }
                            } else if let Some(ref country) = qrz.country {
                                self.new_contact.qth = country.clone();
                            }
                            if let Some(ref addr2) = qrz.addr2 {
                                self.new_contact.city = addr2.clone();
                            }
                            if let Some(ref state) = qrz.state {
                                self.new_contact.state = state.clone();
                            }
                            if let Some(ref grid) = qrz.grid {
                                self.new_contact.grid_square = grid.clone();
                            }
                            self.status_message =
                                format!("Found: {} - {}", qrz.call, self.new_contact.name);
                        }
                        _ => {
                            self.status_message = format!("Lookup error: {}", e);
                        }
                    }
                } else {
                    self.status_message = format!("Lookup error: {}", e);
                }
            }
        }
    }

    fn rig_connect(&mut self) {
        let config = self.rig_config.clone();
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        match rt.block_on(async {
            let mut client = RigCtlClient::new(config);
            client.connect().await?;
            client.update_state().await;
            Ok::<_, String>(client)
        }) {
            Ok(client) => {
                self.rig_client = Some(client);
                self.rig_status_message = Some("Connected to rig".to_string());
                self.status_message = "Connected to transceiver".to_string();
                
                if let Some(ref client) = self.rig_client {
                    let freq = client.state().frequency;
                    if freq > 0.0 {
                        self.new_contact.frequency = freq;
                        if let Some(band) = crate::models::frequency_to_band(freq) {
                            self.new_contact.band = band.to_string();
                        }
                    }
                }
            }
            Err(e) => {
                self.rig_status_message = Some(format!("Connection failed: {}", e));
                self.status_message = format!("Rig connection failed: {}", e);
            }
        }
    }

    fn rig_disconnect(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Some(ref mut client) = self.rig_client {
            let _ = rt.block_on(client.disconnect());
        }
        self.rig_client = None;
        self.rig_status_message = Some("Disconnected".to_string());
        self.status_message = "Disconnected from transceiver".to_string();
    }

    fn rig_update_state(&mut self) {
        if let Some(ref mut client) = self.rig_client {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(client.update_state());
            
            let freq = client.state().frequency;
            if freq > 0.0 && (self.new_contact.frequency - freq).abs() > 0.001 {
                self.new_contact.frequency = freq;
                if let Some(band) = crate::models::frequency_to_band(freq) {
                    self.new_contact.band = band.to_string();
                }
            }
            
            let mode = &client.state().mode;
            if !mode.is_empty() {
                let mode_str = crate::rigctl::mode_to_string(mode);
                if mode_str != "Unknown" {
                    self.new_contact.mode = mode_str.to_string();
                }
            }
        }
    }

    fn is_rig_connected(&self) -> bool {
        self.rig_client.as_ref().map(|c| c.state().connected).unwrap_or(false)
    }

    fn check_lotw_sync(&mut self, ctx: &egui::Context) {
        if self.sync_in_progress {
            return;
        }
        if self.lotw_client.is_none() {
            return;
        }

        if self.last_sync_instant.elapsed().as_secs() >= SYNC_INTERVAL_SECS {
            self.do_lotw_sync();
            ctx.request_repaint();
        }
    }

    fn do_lotw_sync(&mut self) {
        if self.lotw_client.is_none() {
            return;
        }

        self.sync_in_progress = true;
        self.sync_error = None;
        self.status_message = "Syncing with LoTW...".to_string();

        let client = self.lotw_client.as_ref().unwrap();
        let sync_state = self.credential_store.load_lotw_sync_state();

        match client.fetch_confirmations(sync_state.last_qsl_timestamp.as_deref()) {
            Ok(result) => {
                if result.is_html_error {
                    self.sync_error = Some("LoTW returned an error. Check your credentials.".to_string());
                    self.status_message = "LoTW sync failed: HTML response".to_string();
                } else {
                    let mut confirmed_count = 0;
                    for record in &result.records {
                        if record.qsl_rcvd.to_uppercase() == "Y" {
                            if let Err(e) = self.db.update_lotw_confirmed(
                                &record.call,
                                &record.qso_date,
                                &record.time_on,
                                &record.band,
                                &record.mode,
                            ) {
                                log::warn!("Failed to update LoTW confirmed: {}", e);
                            } else {
                                confirmed_count += 1;
                            }
                        }
                    }

                    let now = Utc::now();
                    let new_state = LotwSyncState {
                        last_qsl_timestamp: result.last_qsl_timestamp.clone(),
                        last_qsorx_timestamp: result.last_qsorx_timestamp.clone(),
                        last_sync: Some(now.format("%Y-%m-%d %H:%M UTC").to_string()),
                    };
                    let _ = self.credential_store.save_lotw_sync_state(&new_state);

                    self.last_sync_time = new_state.last_sync.clone();
                    self.last_sync_instant = std::time::Instant::now();
                    self.status_message = if confirmed_count > 0 {
                        format!("LoTW sync: {} new confirmations", confirmed_count)
                    } else {
                        format!("LoTW sync complete: {} records checked", result.records.len())
                    };
                    self.refresh_contacts();
                }
            }
            Err(e) => {
                self.sync_error = Some(e.clone());
                self.status_message = format!("LoTW sync failed: {}", e);
            }
        }

        self.sync_in_progress = false;
    }

    fn submit_to_lotw(&mut self) {
        if !self.station_profile.is_complete() {
            self.lotw_submit_error = Some(
                "Station callsign and grid square are required before submitting to LoTW.".to_string(),
            );
            self.lotw_submit_success = None;
            return;
        }

        let unsubmitted = match self.db.get_unsubmitted_contacts() {
            Ok(c) => c,
            Err(e) => {
                self.lotw_submit_error = Some(format!("Failed to query unsubmitted contacts: {}", e));
                self.lotw_submit_success = None;
                return;
            }
        };

        if unsubmitted.is_empty() {
            self.lotw_submit_error = Some("No unsubmitted contacts found.".to_string());
            self.lotw_submit_success = None;
            return;
        }

        let mut missing_required = Vec::new();
        let mut missing_station = Vec::new();

        for contact in &unsubmitted {
            let missing = contact.can_submit_to_lotw();
            if !missing.is_empty() {
                missing_required.push(format!("{}: {}", contact.call_sign, missing.join(", ")));
            }
            let station_missing = contact.missing_lotw_station_fields();
            if !station_missing.is_empty() {
                missing_station.push(format!("{}: {}", contact.call_sign, station_missing.join(", ")));
            }
        }

        let mut errors = Vec::new();
        if !missing_required.is_empty() {
            errors.push(format!("Missing required fields:\n{}", missing_required.join("\n")));
        }
        if !missing_station.is_empty() {
            errors.push(format!("Missing station fields:\n{}", missing_station.join("\n")));
        }

        if !errors.is_empty() {
            self.lotw_submit_error = Some(errors.join("\n\n"));
            self.lotw_submit_success = None;
            return;
        }

        let filename = lotw::generate_lotw_filename();
        let path = std::path::PathBuf::from(&filename);

        match lotw::export_for_lotw(&unsubmitted, &self.station_profile, &path) {
            Ok(count) => {
                let ids: Vec<i64> = unsubmitted.iter().filter_map(|c| c.id).collect();
                let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                if let Err(e) = self.db.mark_submitted(&ids, &now) {
                    log::warn!("Failed to mark contacts as submitted: {}", e);
                }

                let _ = open_file(&path);

                self.lotw_submit_success = Some(format!(
                    "Exported {} contacts to {} — marked as submitted",
                    count, filename
                ));
                self.lotw_submit_error = None;
                self.refresh_contacts();
            }
            Err(e) => {
                self.lotw_submit_error = Some(format!("Export failed: {}", e));
                self.lotw_submit_success = None;
            }
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let visuals = match self.current_theme.as_str() {
            "Dark" => egui::Visuals::dark(),
            "Light" => egui::Visuals::light(),
            "Dark Blue" => {
                let mut v = egui::Visuals::dark();
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x1e, 0x3e, 0x5e);
                v.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(0x2d, 0x5d, 0x8d);
                v
            }
            "Monokai" => {
                let mut v = egui::Visuals::dark();
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x27, 0x28, 0x22);
                v.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(0xf8, 0xf8, 0xf2);
                v
            }
            "One Dark" => {
                let mut v = egui::Visuals::dark();
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x28, 0x2c, 0x34);
                v.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(0x3e, 0x44, 0x52);
                v
            }
            "Solarized Dark" => {
                let mut v = egui::Visuals::dark();
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x00, 0x2b, 0x36);
                v.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(0x83, 0x96, 0xa5);
                v
            }
            "Solarized Light" => {
                let mut v = egui::Visuals::light();
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0xf6, 0xf6, 0xe4);
                v.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(0x65, 0x7b, 0x8a);
                v
            }
            "Dracula" => {
                let mut v = egui::Visuals::dark();
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x28, 0x29, 0x33);
                v.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(0x62, 0x4e, 0x69);
                v
            }
            "Gruvbox Dark" => {
                let mut v = egui::Visuals::dark();
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x28, 0x20, 0x18);
                v.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(0xeb, 0xdb, 0xb2);
                v
            }
            "Nord" => {
                let mut v = egui::Visuals::dark();
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x2e, 0x34, 0x40);
                v.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(0x4c, 0x56, 0x6a);
                v
            }
            _ => egui::Visuals::dark(),
        };
        ctx.set_visuals(visuals);
    }
}

fn open_file(path: &std::path::Path) {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn().ok();
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path.to_str().unwrap_or("")])
            .spawn()
            .ok();
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .ok();
    }
}

fn format_utc_time(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%H:%M:%S").to_string()
}

fn format_local_time(dt: &chrono::DateTime<chrono::Local>) -> String {
    dt.format("%H:%M:%S").to_string()
}

impl eframe::App for QsoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_counter += 1;
        self.rig_update_state();
        self.check_lotw_sync(ctx);
        
        let utc_now = Utc::now();
        let local_now = Local::now();
        
        if self.frame_counter % 60 == 0 || self.frame_counter == 1 {
            if self.new_contact.qso_date.is_empty() || self.new_contact.qso_time.is_empty() {
                self.new_contact.qso_date = utc_now.format("%Y-%m-%d").to_string();
                self.new_contact.qso_time = local_now.format("%H%M%S").to_string();
            }
        }
        
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("QSOLink - Ham Radio Logger");
                
                ui.separator();
                
                let is_connected = self.is_rig_connected();
                if is_connected {
                    if let Some(ref client) = self.rig_client {
                        let freq_display = client.format_frequency();
                        ui.colored_label(egui::Color32::GREEN, format!("\u{25CF} {}", freq_display));
                    }
                } else {
                    ui.colored_label(egui::Color32::RED, "\u{25CB} Disconnected");
                }
                
                ui.separator();
                
                ui.colored_label(egui::Color32::GREEN, format!("UTC {}", format_utc_time(&utc_now)));
                ui.label(format!("(Local: {})", format_local_time(&local_now)));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("hamburger_menu")
                        .selected_text("\u{2630}")
                        .show_ui(ui, |ui| {
                            if ui.button("Export ADIF").clicked() {
                                self.export_adif();
                            }
                            if ui.button("Export Cabrillo").clicked() {
                                self.export_cabrillo();
                            }
                            ui.separator();
                            if ui.button("LoTW Settings").clicked() {
                                self.station_profile_backup = self.station_profile.clone();
                                self.lotw_username_backup = self.lotw_username.clone();
                                self.lotw_password_backup = self.lotw_password.clone();
                                self.show_lotw_settings = true;
                            }
                            if ui.button("QRZ Settings").clicked() {
                                self.qrz_username_backup = self.qrz_username.clone();
                                self.qrz_password_backup = self.qrz_password.clone();
                                self.show_qrz_settings = true;
                            }
                            if ui.button("Database Settings").clicked() {
                                self.show_db_settings = true;
                            }
                            if ui.button("Rig Settings").clicked() {
                                self.show_rig_settings = true;
                            }
                            ui.separator();
                            if ui.button("Color Theme").clicked() {
                                self.show_theme_settings = true;
                            }
                        });
                });
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status_message);
        });

        if self.show_qrz_settings {
            egui::Window::new("QRZ.com Settings")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("QRZ.com Credentials");
                    ui.label("(Credentials are encrypted on disk)");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut self.qrz_username);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.add(egui::TextEdit::singleline(&mut self.qrz_password).password(true));
                    });

                    ui.horizontal(|ui| {
                        if self.credential_store.has_credentials() {
                            ui.label("Credentials saved securely");
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save & Close").clicked() {
                            let username_changed = self.qrz_username != self.qrz_username_backup;

                            self.show_qrz_settings = false;

                            if !self.qrz_username.is_empty() && !self.qrz_password.is_empty() {
                                if username_changed {
                                    match self
                                        .credential_store
                                        .save_credentials(&self.qrz_username, &self.qrz_password)
                                    {
                                        Ok(_) => {
                                            log::info!("New QRZ credentials saved");
                                        }
                                        Err(e) => {
                                            log::error!("Failed to save credentials: {}", e);
                                            self.status_message =
                                                format!("Failed to save credentials: {}", e);
                                            return;
                                        }
                                    }
                                }

                                self.qrz_client = Some(crate::qrz::QrzClient::new(
                                    self.qrz_username.clone(),
                                    self.qrz_password.clone(),
                                ));
                                self.status_message =
                                    "QRZ credentials configured - Ready to lookup".to_string();
                            } else if self.qrz_username.is_empty() && self.qrz_password.is_empty() {
                                let _ = self.credential_store.delete_credentials();
                                self.qrz_client = None;
                                self.status_message = "QRZ credentials cleared".to_string();
                            } else {
                                self.status_message = "Username and password required".to_string();
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.qrz_username = self.qrz_username_backup.clone();
                            self.qrz_password = self.qrz_password_backup.clone();
                            self.show_qrz_settings = false;
                            self.status_message = "QRZ settings unchanged".to_string();
                        }
                    });
                });
        }

        if self.show_db_settings {
            egui::Window::new("Database Settings")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Database Configuration");
                    ui.separator();

                    let is_connected = self
                        .remote_db
                        .as_ref()
                        .map(|r| r.is_connected())
                        .unwrap_or(false);

                    if is_connected {
                        ui.label("Status: Connected to remote database");
                    } else if self.use_remote_db {
                        ui.label("Status: Using local SQLite database");
                    } else {
                        ui.label("Status: Using local SQLite database");
                    }

                    ui.separator();
                    ui.label("Database Type:");
                    egui::ComboBox::from_id_salt("db_type_combo")
                        .selected_text(format!("{:?}", self.db_config.db_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.db_config.db_type,
                                DatabaseType::SQLite,
                                "SQLite (Local)",
                            );
                            ui.selectable_value(
                                &mut self.db_config.db_type,
                                DatabaseType::PostgreSQL,
                                "PostgreSQL",
                            );
                            ui.selectable_value(
                                &mut self.db_config.db_type,
                                DatabaseType::MySQL,
                                "MySQL",
                            );
                        });

                    ui.label("Connection String:");
                    ui.text_edit_singleline(&mut self.db_config.connection_string);

                    ui.label("Example:");
                    ui.label(RemoteDatabase::connection_string_example(&self.db_config.db_type));

                    if let Some(ref result) = self.connection_test_result {
                        ui.separator();
                        ui.label(result);
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        if !is_connected {
                            if ui.button("Test Connection").clicked() {
                                let config = self.db_config.clone();
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                self.connection_test_result = Some("Testing connection...".to_string());
                                match rt.block_on(async {
                                    let test_db = RemoteDatabase::new(config);
                                    test_db.test_connection().await
                                }) {
                                    Ok(true) => {
                                        self.connection_test_result =
                                            Some("Connection successful!".to_string());
                                    }
                                    Ok(false) => {
                                        self.connection_test_result =
                                            Some("Connection failed".to_string());
                                    }
                                    Err(e) => {
                                        self.connection_test_result =
                                            Some(format!("Error: {}", e));
                                    }
                                }
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        if !is_connected {
                            if ui.button("Connect").clicked() {
                                let config = self.db_config.clone();
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                match rt.block_on(async {
                                    let mut remote_db = RemoteDatabase::new(config);
                                    remote_db.connect().await?;
                                    remote_db.create_table_if_not_exists().await?;
                                    Ok::<_, String>(remote_db)
                                }) {
                                    Ok(remote_db) => {
                                        self.remote_db = Some(remote_db);
                                        self.use_remote_db = true;
                                        self.connection_test_result =
                                            Some("Connected successfully!".to_string());
                                        self.status_message =
                                            "Connected to remote database".to_string();
                                        self.refresh_contacts();
                                    }
                                    Err(e) => {
                                        self.connection_test_result =
                                            Some(format!("Connection failed: {}", e));
                                        self.status_message =
                                            format!("Connection failed: {}", e);
                                    }
                                }
                            }
                        } else {
                            if ui.button("Disconnect").clicked() {
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                if let Some(ref mut remote_db) = self.remote_db {
                                    let _ = rt.block_on(remote_db.disconnect());
                                }
                                self.remote_db = None;
                                self.use_remote_db = false;
                                self.connection_test_result =
                                    Some("Disconnected from remote database".to_string());
                                self.status_message = "Disconnected from remote database".to_string();
                                self.refresh_contacts();
                            }
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save & Close").clicked() {
                            self.show_db_settings = false;
                            self.connection_test_result = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_db_settings = false;
                            self.connection_test_result = None;
                            self.status_message = "Database settings unchanged".to_string();
                        }
                    });
                });
        }

        if self.show_rig_settings {
            egui::Window::new("Transceiver Settings")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Hamlib rigctld Connection");
                    ui.label("(Requires rigctld running on localhost)");
                    ui.separator();

                    let is_connected = self.is_rig_connected();
                    
                    if is_connected {
                        ui.colored_label(egui::Color32::GREEN, "\u{25CF} Connected");
                        if let Some(ref client) = self.rig_client {
                            ui.label(format!("Frequency: {}", client.format_frequency()));
                            ui.label(format!("Mode: {}", client.state().mode));
                        }
                    } else {
                        ui.colored_label(egui::Color32::RED, "\u{25CB} Disconnected");
                    }

                    if let Some(ref msg) = self.rig_status_message {
                        ui.separator();
                        ui.label(msg);
                    }

                    ui.separator();
                    ui.label("Host:");
                    ui.text_edit_singleline(&mut self.rig_config.host);

                    ui.label("Port:");
                    ui.add(egui::DragValue::new(&mut self.rig_config.port).range(1024..=65535));

                    ui.label("Poll Interval (ms):");
                    ui.add(egui::DragValue::new(&mut self.rig_config.poll_interval_ms).range(100..=5000));

                    ui.label("Example rigctld command:");
                    ui.label("rigctld -m 1024 -r /dev/ttyUSB0 -s 115200 -T localhost -t 4532");
                    ui.label("(1024 = Icom IC-7300, adjust for your radio)");

                    ui.separator();
                    ui.horizontal(|ui| {
                        if !is_connected {
                            if ui.button("Connect").clicked() {
                                self.rig_connect();
                            }
                        } else {
                            if ui.button("Disconnect").clicked() {
                                self.rig_disconnect();
                            }
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            self.show_rig_settings = false;
                            self.rig_status_message = None;
                        }
                    });
                });
        }

        if self.show_theme_settings {
            egui::Window::new("Color Theme")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Select Color Theme");
                    ui.separator();
                    
                    let themes = vec![
                        "Dark",
                        "Light", 
                        "Dark Blue",
                        "Monokai",
                        "One Dark",
                        "Solarized Dark",
                        "Solarized Light",
                        "Dracula",
                        "Gruvbox Dark",
                        "Nord",
                    ];
                    
                    for theme in themes {
                        let is_selected = self.current_theme == theme;
                        if ui.selectable_label(is_selected, theme).clicked() {
                            self.current_theme = theme.to_string();
                            self.apply_theme(ctx);
                        }
                    }
                    
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            self.show_theme_settings = false;
                        }
                    });
                });
        }

        if self.show_lotw_settings {
            egui::Window::new("LoTW Settings")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("LoTW Settings");
                    ui.separator();

                    ui.group(|ui| {
                        ui.label("LoTW Account");
                        ui.label("(Same credentials as lotw.arrl.org)");
                        ui.horizontal(|ui| {
                            ui.label("Username:");
                            ui.text_edit_singleline(&mut self.lotw_username);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Password:");
                            ui.add(egui::TextEdit::singleline(&mut self.lotw_password).password(true));
                        });
                    });

                    ui.separator();

                    ui.group(|ui| {
                        ui.label("My Station (Required for LoTW)");
                        ui.horizontal(|ui| {
                            ui.label("My Callsign:");
                            ui.text_edit_singleline(&mut self.station_profile.callsign);
                            ui.label(" *");
                        });
                        ui.horizontal(|ui| {
                            ui.label("My Grid Square:");
                            ui.text_edit_singleline(&mut self.station_profile.grid_square);
                            ui.label(" * (e.g. EM75, EM75AX)");
                        });
                        ui.horizontal(|ui| {
                            ui.label("CQ Zone:");
                            ui.add(egui::DragValue::new(&mut self.station_profile.cq_zone).range(1..=40));
                        });
                        ui.horizontal(|ui| {
                            ui.label("ITU Zone:");
                            ui.add(egui::DragValue::new(&mut self.station_profile.itu_zone).range(1..=90));
                        });
                        ui.horizontal(|ui| {
                            ui.label("ARRL Section:");
                            ui.text_edit_singleline(&mut self.station_profile.arl_section);
                        });
                    });

                    ui.separator();

                    ui.group(|ui| {
                        ui.label("Sync");
                        if let Some(ref last) = self.last_sync_time {
                            ui.label(format!("Last sync: {}", last));
                        } else {
                            ui.label("Never synced");
                        }
                        if self.sync_in_progress {
                            ui.colored_label(egui::Color32::YELLOW, "Syncing...");
                        }
                        if let Some(ref err) = self.sync_error {
                            ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                        }
                        if ui.button("Sync Now").clicked() {
                            if self.lotw_client.is_none() && !self.lotw_username.is_empty() && !self.lotw_password.is_empty() {
                                self.lotw_client = Some(LotwClient::new(self.lotw_username.clone(), self.lotw_password.clone()));
                            }
                            self.do_lotw_sync();
                        }
                    });

                    ui.separator();

                    ui.group(|ui| {
                        ui.label("Submission");
                        let unsubmitted_count = self.db.get_unsubmitted_contacts().map(|c| c.len()).unwrap_or(0);
                        ui.label(format!("Unsubmitted contacts: {}", unsubmitted_count));
                        if let Some(ref err) = self.lotw_submit_error {
                            ui.colored_label(egui::Color32::RED, err.clone());
                        }
                        if let Some(ref msg) = self.lotw_submit_success {
                            ui.colored_label(egui::Color32::GREEN, msg.clone());
                        }
                        if ui.button("Submit to LoTW").clicked() {
                            self.submit_to_lotw();
                        }
                        ui.label("(Generates ADIF file for signing with TQSL)");
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save & Close").clicked() {
                            if let Err(e) = self.credential_store.save_station_profile(&self.station_profile) {
                                log::error!("Failed to save station profile: {}", e);
                            }

                            if !self.lotw_username.is_empty() && !self.lotw_password.is_empty() {
                                if self.lotw_username != self.lotw_username_backup || self.lotw_password != self.lotw_password_backup {
                                    if let Err(e) = self.credential_store.save_lotw_credentials(&self.lotw_username, &self.lotw_password) {
                                        log::error!("Failed to save LoTW credentials: {}", e);
                                    }
                                    self.lotw_client = Some(LotwClient::new(self.lotw_username.clone(), self.lotw_password.clone()));
                                }
                            } else {
                                let _ = self.credential_store.delete_lotw_credentials();
                                self.lotw_client = None;
                            }

                            self.show_lotw_settings = false;
                            self.status_message = "LoTW settings saved".to_string();
                        }
                        if ui.button("Cancel").clicked() {
                            self.station_profile = self.station_profile_backup.clone();
                            self.lotw_username = self.lotw_username_backup.clone();
                            self.lotw_password = self.lotw_password_backup.clone();
                            self.show_lotw_settings = false;
                            self.lotw_submit_error = None;
                            self.lotw_submit_success = None;
                            self.status_message = "LoTW settings unchanged".to_string();
                        }
                    });
                });
        }

        egui::SidePanel::left("input_panel")
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("New Contact");
                ui.separator();

                ui.label("Call Sign *");
                ui.horizontal(|ui| {
                    let call_response = ui.text_edit_singleline(&mut self.new_contact.call_sign);
                    if call_response.changed() {
                        let query = self.new_contact.call_sign.to_lowercase();
                        if query.len() >= 1 {
                            self.callsign_suggestions = self.contacts.iter()
                                .filter(|c| c.call_sign.to_lowercase().starts_with(&query))
                                .take(5)
                                .cloned()
                                .collect();
                        } else {
                            self.callsign_suggestions.clear();
                        }
                    }
                    if ui.button("QRZ").clicked() {
                        self.do_lookup();
                    }
                });
                
                if !self.callsign_suggestions.is_empty() {
                    ui.label("Suggestions:");
                    let suggestions: Vec<Contact> = self.callsign_suggestions.clone();
                    for suggestion in &suggestions {
                        let label = format!("{} - {} ({})", suggestion.call_sign, suggestion.name, suggestion.qth);
                        if ui.selectable_label(false, &label).clicked() {
                            self.new_contact.call_sign = suggestion.call_sign.clone();
                            self.new_contact.name = suggestion.name.clone();
                            self.new_contact.qth = suggestion.qth.clone();
                            self.new_contact.band = suggestion.band.clone();
                            self.new_contact.mode = suggestion.mode.clone();
                            self.new_contact.frequency = suggestion.frequency;
                            self.new_contact.city = suggestion.city.clone();
                            self.new_contact.state = suggestion.state.clone();
                            self.new_contact.grid_square = suggestion.grid_square.clone();
                            self.callsign_suggestions.clear();
                        }
                    }
                }

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut self.new_contact.name);
                    });
                    ui.vertical(|ui| {
                        ui.label("QTH");
                        ui.text_edit_singleline(&mut self.new_contact.qth);
                    });
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Band");
                        egui::ComboBox::from_id_salt("band_combo")
                            .selected_text(&self.new_contact.band)
                            .show_ui(ui, |ui| {
                                for band in BANDS {
                                    ui.selectable_value(
                                        &mut self.new_contact.band,
                                        band.to_string(),
                                        *band,
                                    );
                                }
                            });
                    });
                    ui.vertical(|ui| {
                        ui.label("Mode *");
                        let needs_warn = mode_needs_warning(&self.new_contact.mode);
                        
                        egui::ComboBox::from_id_salt("mode_combo")
                            .selected_text(&self.new_contact.mode)
                            .show_ui(ui, |ui| {
                                let common_modes = ["CW", "SSB", "USB", "LSB", "FM", "AM", "RTTY", "DATA"];
                                for m in &common_modes {
                                    ui.selectable_value(&mut self.new_contact.mode, (*m).to_string(), *m);
                                }
                                ui.separator();
                                ui.label("Digital Modes:");
                                let digital_modes = ["FT8", "FT4", "PSK31", "PSK63", "JS8", "MFSK", "OLIVIA"];
                                for m in &digital_modes {
                                    ui.selectable_value(&mut self.new_contact.mode, (*m).to_string(), *m);
                                }
                                ui.separator();
                                if ui.selectable_label(self.use_custom_mode, "Other...").clicked() {
                                    self.use_custom_mode = true;
                                    self.mode_custom_text = self.new_contact.mode.clone();
                                }
                            });
                        
                        if needs_warn {
                            ui.colored_label(egui::Color32::YELLOW, "FT8/FT4 are digital submodes");
                        }
                    });
                });

                if self.use_custom_mode {
                    ui.horizontal(|ui| {
                        ui.label("Custom Mode:");
                        ui.text_edit_singleline(&mut self.mode_custom_text);
                        if ui.button("Use").clicked() {
                            self.new_contact.mode = self.mode_custom_text.clone();
                            self.use_custom_mode = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.use_custom_mode = false;
                            self.mode_custom_text.clear();
                        }
                    });
                }

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Frequency (MHz)");
                        ui.add(egui::DragValue::new(&mut self.new_contact.frequency).speed(0.001));
                    });
                    ui.vertical(|ui| {
                        ui.label("RST Sent");
                        ui.text_edit_singleline(&mut self.new_contact.rst_sent);
                    });
                    ui.vertical(|ui| {
                        ui.label("RST Recv");
                        ui.text_edit_singleline(&mut self.new_contact.rst_recv);
                    });
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Date (YYYY-MM-DD) *");
                        ui.text_edit_singleline(&mut self.new_contact.qso_date);
                    });
                    ui.vertical(|ui| {
                        ui.label("Time (UTC, HHMMSS) *");
                        ui.text_edit_singleline(&mut self.new_contact.qso_time);
                    });
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Grid Square");
                        ui.text_edit_singleline(&mut self.new_contact.grid_square);
                    });
                    ui.vertical(|ui| {
                        ui.label("CQ Zone");
                        ui.add(egui::DragValue::new(&mut self.new_contact.cq_zone).range(1..=40));
                    });
                    ui.vertical(|ui| {
                        ui.label("ITU Zone");
                        ui.add(egui::DragValue::new(&mut self.new_contact.itu_zone).range(1..=90));
                    });
                });

                if !self.new_contact.grid_square.trim().is_empty() {
                    if validate_grid_square(&self.new_contact.grid_square).is_err() {
                        ui.colored_label(egui::Color32::RED, "Invalid grid square format");
                    }
                }

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("City");
                        ui.text_edit_singleline(&mut self.new_contact.city);
                    });
                    ui.vertical(|ui| {
                        ui.label("State");
                        ui.text_edit_singleline(&mut self.new_contact.state);
                    });
                });

                ui.label("Notes");
                ui.text_edit_multiline(&mut self.new_contact.notes);

                ui.separator();

                let missing = self.new_contact.can_submit_to_lotw();
                if missing.is_empty() {
                    ui.colored_label(egui::Color32::GREEN, "✓ Ready for LoTW");
                } else {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("✗ Missing for LoTW: {}", missing.join(", ")),
                    );
                }

                if ui.button("Add Contact").clicked() {
                    self.add_contact();
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Contacts");
            ui.horizontal(|ui| {
                ui.label("Search:");
                let search_changed = ui.text_edit_singleline(&mut self.search_query).changed();
                if search_changed {
                    self.refresh_contacts();
                }
                if ui.button("Clear").clicked() {
                    self.search_query.clear();
                    self.refresh_contacts();
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("contacts_grid")
                    .num_columns(9)
                    .spacing([10.0, 5.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Date");
                        ui.label("Time");
                        ui.label("Call");
                        ui.label("Name");
                        ui.label("Band");
                        ui.label("Mode");
                        ui.label("RST");
                        ui.label("QSOs");
                        ui.label("LoTW");
                        ui.end_row();

                        for contact in &self.contacts {
                            ui.label(&contact.qso_date);
                            ui.label(&contact.qso_time);
                            let call_response = ui.selectable_label(false, &contact.call_sign);
                            if call_response.clicked() {
                                self.show_callsign_details = Some(contact.clone());
                            }
                            ui.label(&contact.name);
                            ui.label(&contact.band);
                            ui.label(&contact.mode);

                            let rst = format!("{} / {}", contact.rst_sent, contact.rst_recv);
                            ui.label(rst);
                            if contact.qso_count > 1 {
                                ui.colored_label(egui::Color32::GOLD, format!("{}", contact.qso_count));
                            } else {
                                ui.label("");
                            }

                            match contact.lotw_status() {
                                LotwStatus::Confirmed => {
                                    ui.colored_label(egui::Color32::GREEN, "✓");
                                }
                                LotwStatus::Submitted => {
                                    ui.colored_label(egui::Color32::YELLOW, "✓");
                                }
                                LotwStatus::NotSubmitted => {
                                    ui.label("-");
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
        });

        if let Some(contact) = self.show_callsign_details.clone() {
            let contact = contact;
            egui::Window::new("Callsign Details")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading(&contact.call_sign);
                    ui.separator();
                    ui.label(format!("Name: {}", contact.name));
                    ui.label(format!("QTH: {}", contact.qth));
                    if !contact.city.is_empty() {
                        ui.label(format!("City: {}", contact.city));
                    }
                    if !contact.state.is_empty() {
                        ui.label(format!("State: {}", contact.state));
                    }
                    if !contact.county.is_empty() {
                        ui.label(format!("County: {}", contact.county));
                    }
                    if !contact.grid_square.is_empty() {
                        ui.label(format!("Grid: {}", contact.grid_square));
                    }
                    ui.label(format!("Band: {}", contact.band));
                    ui.label(format!("Mode: {}", contact.mode));
                    ui.label(format!("Frequency: {:.3} MHz", contact.frequency));
                    ui.label(format!("RST Sent: {}", contact.rst_sent));
                    ui.label(format!("RST Recv: {}", contact.rst_recv));
                    ui.label(format!("Date: {}", contact.qso_date));
                    ui.label(format!("Time: {}", contact.qso_time));
                    ui.label(format!("Total QSOs with {}: {}", contact.call_sign, contact.qso_count));
                    
                    ui.separator();
                    ui.label("LoTW Status:");
                    match contact.lotw_status() {
                        LotwStatus::Confirmed => {
                            ui.colored_label(egui::Color32::GREEN, "Confirmed");
                        }
                        LotwStatus::Submitted => {
                            ui.colored_label(egui::Color32::YELLOW, "Submitted (pending confirmation)");
                        }
                        LotwStatus::NotSubmitted => {
                            ui.colored_label(egui::Color32::GRAY, "Not submitted");
                        }
                    }
                    if let Some(ref date) = contact.lotw_submitted_date {
                        ui.label(format!("Submitted: {}", date));
                    }
                    
                    if !contact.notes.is_empty() {
                        ui.separator();
                        ui.label(format!("Notes: {}", contact.notes));
                    }
                    
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            self.show_callsign_details = None;
                        }
                    });
                });
        }

        if self.show_delete_confirm {
            egui::Window::new("Confirm Delete")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to delete this contact?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes, Delete").clicked() {
                            if let Some(id) = self.contact_to_delete {
                                self.delete_contact(id);
                            }
                            self.show_delete_confirm = false;
                            self.contact_to_delete = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_delete_confirm = false;
                            self.contact_to_delete = None;
                        }
                    });
                });
        }
    }
}
