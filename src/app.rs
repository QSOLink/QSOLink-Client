use crate::db::Database;
use crate::models::{validate_callsign, Contact, BANDS, MODES};
use eframe::egui;

pub struct QsoApp {
    db: Database,
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
    db_config: crate::remote_db::DatabaseConfig,
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
            contacts,
            search_query: String::new(),
            new_contact,
            selected_contact: None,
            status_message: String::from("Ready - Click 'QRZ Settings' to configure lookup"),
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
            db_config: crate::remote_db::DatabaseConfig::default(),
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
}

impl eframe::App for QsoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    ui.label("(Remote database support coming soon)");
                    ui.separator();

                    ui.label("Database Type:");
                    egui::ComboBox::from_id_salt("db_type_combo")
                        .selected_text(format!("{:?}", self.db_config.db_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.db_config.db_type,
                                crate::remote_db::DatabaseType::SQLite,
                                "SQLite (Local)",
                            );
                            ui.selectable_value(
                                &mut self.db_config.db_type,
                                crate::remote_db::DatabaseType::PostgreSQL,
                                "PostgreSQL",
                            );
                            ui.selectable_value(
                                &mut self.db_config.db_type,
                                crate::remote_db::DatabaseType::MySQL,
                                "MySQL",
                            );
                        });

                    ui.label("Connection String:");
                    ui.text_edit_singleline(&mut self.db_config.connection_string);

                    ui.label("Example:");
                    ui.label(crate::remote_db::RemoteDatabase::connection_string_example(
                        &self.db_config.db_type,
                    ));

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.show_db_settings = false;
                            self.status_message =
                                "Database config saved (remote DB coming soon)".to_string();
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_db_settings = false;
                            self.status_message = "Database settings unchanged".to_string();
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
