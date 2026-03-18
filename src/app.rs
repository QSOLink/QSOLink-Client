use crate::db::Database;
use crate::models::{validate_callsign, Contact, BANDS, MODES};
use crate::remote_db::{DatabaseConfig, DatabaseType, RemoteDatabase};
use crate::rigctl::{RigCtlClient, RigConfig};
use eframe::egui;

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
    credential_store: crate::security::CredentialStore,
    show_db_settings: bool,
    db_config: DatabaseConfig,
    use_remote_db: bool,
    connection_test_result: Option<String>,
    show_rig_settings: bool,
    rig_client: Option<RigCtlClient>,
    rig_config: RigConfig,
    rig_status_message: Option<String>,
}

impl QsoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, db_path: Option<std::path::PathBuf>) -> Self {
        env_logger::init();

        let db = Database::new(db_path).expect("Failed to initialize database");
        let contacts = db.get_all_contacts().unwrap_or_default();
        let new_contact = Contact::new(String::new());

        let credential_store = crate::security::CredentialStore::new();
        let (qrz_username, qrz_password) = credential_store
            .load_credentials()
            .unwrap_or((String::new(), String::new()));

        log::info!(
            "QSOLink started - QRZ credentials loaded: {}",
            !qrz_username.is_empty()
        );

        Self {
            db,
            remote_db: None,
            contacts,
            search_query: String::new(),
            new_contact,
            selected_contact: None,
            status_message: String::from("Ready - Click 'QRZ Settings' to configure lookup"),
            show_delete_confirm: false,
            contact_to_delete: None,
            qrz_client: None,
            qrz_username,
            qrz_username_backup: String::new(),
            qrz_password_backup: String::new(),
            show_qrz_settings: false,
            credential_store,
            show_db_settings: false,
            db_config: DatabaseConfig::default(),
            use_remote_db: false,
            connection_test_result: None,
            qrz_password,
            show_rig_settings: false,
            rig_client: None,
            rig_config: RigConfig::default(),
            rig_status_message: None,
        }
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
        self.contacts = if self.search_query.is_empty() {
            self.db.get_all_contacts().unwrap_or_default()
        } else {
            self.db
                .search_contacts(&self.search_query)
                .unwrap_or_default()
        };
    }

    fn search(&mut self) {
        self.refresh_contacts();
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
}

impl eframe::App for QsoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.rig_update_state();
        
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("QSOLink - Ham Radio Logger");
                ui.separator();
                if ui.button("Export ADIF").clicked() {
                    self.export_adif();
                }
                if ui.button("Export Cabrillo").clicked() {
                    self.export_cabrillo();
                }
                ui.separator();
                if ui.button("QRZ Settings").clicked() {
                    self.qrz_username_backup = self.qrz_username.clone();
                    self.qrz_password_backup = self.qrz_password.clone();
                    self.show_qrz_settings = !self.show_qrz_settings;
                }
                ui.separator();
                if ui.button("Database Settings").clicked() {
                    self.show_db_settings = !self.show_db_settings;
                }
                ui.separator();
                if ui.button("Rig Settings").clicked() {
                    self.show_rig_settings = !self.show_rig_settings;
                }
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

        egui::SidePanel::left("input_panel")
            .min_width(350.0)
            .show(ctx, |ui| {
                ui.heading("New Contact");
                ui.separator();

                ui.label("Call Sign *");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.new_contact.call_sign);
                    if ui.button("QRZ Lookup").clicked() {
                        self.do_lookup();
                    }
                });

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
                        ui.label("Mode");
                        egui::ComboBox::from_id_salt("mode_combo")
                            .selected_text(&self.new_contact.mode)
                            .show_ui(ui, |ui| {
                                for mode in MODES {
                                    ui.selectable_value(
                                        &mut self.new_contact.mode,
                                        mode.to_string(),
                                        *mode,
                                    );
                                }
                            });
                    });
                });

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
                        ui.label("Date (YYYY-MM-DD)");
                        ui.text_edit_singleline(&mut self.new_contact.qso_date);
                    });
                    ui.vertical(|ui| {
                        ui.label("Time (HHMM)");
                        ui.text_edit_singleline(&mut self.new_contact.qso_time);
                    });
                });

                ui.label("Notes");
                ui.text_edit_multiline(&mut self.new_contact.notes);

                ui.separator();
                if ui.button("Add Contact").clicked() {
                    self.add_contact();
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Contacts");
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_query);
                if ui.button("Search").clicked() {
                    self.search();
                }
                if ui.button("Clear").clicked() {
                    self.search_query.clear();
                    self.refresh_contacts();
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("contacts_grid")
                    .num_columns(7)
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
                        ui.end_row();

                        for contact in &self.contacts {
                            ui.label(&contact.qso_date);
                            ui.label(&contact.qso_time);
                            ui.label(&contact.call_sign);
                            ui.label(&contact.name);
                            ui.label(&contact.band);
                            ui.label(&contact.mode);

                            let rst = format!("{} / {}", contact.rst_sent, contact.rst_recv);
                            ui.label(rst);
                            ui.end_row();
                        }
                    });
            });
        });

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
