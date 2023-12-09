use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Qso {
    #[serde(rename = "ID")]
    id:           i32,
    #[serde(rename = "CreatedAt")]
    created_at:    String,
    #[serde(rename = "UpdatedAt")]
    updated_at:    String,
    #[serde(rename = "DeletedAt")]
    deleted_at:    Option<String>,
    dateon:       Option<String>,
    timeon:       Option<String>,
    callsign:     Option<String>,
    band:         Option<i32>,
    mode:         Option<String>,
    city:         Option<String>,
    state:        Option<String>,
    county:       Option<String>,
    country:      Option<String>,
    name:         Option<String>,
    qslr:         Option<bool>,
    qsls:         Option<bool>,
    rstr:         Option<i32>,
    rsts:         Option<i32>,
    power:        Option<i32>,
    remarks:      Option<String>
}
