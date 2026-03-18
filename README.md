# QSOLink - Ham Radio Logging Software

A cross-platform ham radio logging application written in Rust using egui.

## Features

- Contact logging with callsign, name, QTH, band, mode, frequency, RST, grid square, notes
- Local SQLite database storage (default)
- Remote database support (PostgreSQL, MySQL)
- QRZ.com callsign lookup with encrypted credential storage
- ARRL Logbook of the World (LoTW) integration — check confirmations, mark submitted/confirmed
- ADIF export (LoTW-compatible with STATION_CALLSIGN, MY_GRIDSQUARE, SUBMODE)
- Cabrillo export (contest format)
- Search and filter contacts
- Transceiver control via Hamlib rigctld (CAT/CIV)
- Cross-platform support (Windows, Linux, macOS)

## Requirements

### All Platforms

- **Rust** (1.70 or later): https://rustup.rs/
- **Cargo** (included with Rust)

### Linux

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Fedora
sudo dnf install gcc pkg-config openssl-devel

# Arch Linux
sudo pacman -S base-devel pkgconf openssl
```

### macOS

- Xcode Command Line Tools:
```bash
xcode-select --install
```

### Windows

- Visual Studio Build Tools or Visual Studio (with C++ workload)
- OR MinGW-w64

## Build Instructions

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/qsolink.git
cd QSOLink-client

# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release
```

### Run

```bash
# Debug mode
cargo run

# Release mode
cargo run --release
```

### Install

```bash
# Linux/macOS
cargo install --path .

# Windows
cargo install --path . --force
```

## Database Configuration

### Local SQLite (Default)

The application automatically creates a `qso.db` file in the current directory.

### Remote Database

1. Click "Database Settings" in the toolbar
2. Select database type (PostgreSQL or MySQL)
3. Enter connection string:
   - PostgreSQL: `postgres://user:password@localhost:5432/qsolog`
   - MySQL: `mysql://user:password@localhost:3306/qsolog`
4. Click "Test Connection" to verify
5. Click "Connect" to switch to remote database

## LoTW Setup

1. Click "LoTW Settings" in the toolbar
2. Enter your LoTW username and password (same as your ARRL account)
3. Enter your station callsign and grid square
4. Click "Save & Close"
5. On app start and every 60 minutes, QSOLink automatically syncs with LoTW to check for confirmations
6. To export contacts for LoTW signing: click "Export ADIF" and open the file in TrustedQSL (TQSL) to apply your digital signature and upload to LoTW

## QRZ.com Setup

1. Click "QRZ Settings" in the toolbar
2. Enter your QRZ.com username and password
3. Click "Save & Close"
4. Credentials are encrypted using AES-256-GCM before storage

## Transceiver Control (CAT/CIV)

### Prerequisites

#### Linux

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install hamlib rigctl

# Fedora
sudo dnf install hamlib rigctl

# Arch Linux
sudo pacman -S hamlib
```

#### macOS

```bash
brew install hamlib
```

#### Windows

Download Hamlib from: https://sourceforge.net/projects/hamlib/files/hamlib/4.7.0/

### Starting rigctld

```bash
# For Icom IC-7300 (USB serial)
rigctld -m 1024 -r /dev/ttyUSB0 -s 115200 -T localhost -t 4532

# For Icom IC-705
rigctld -m 1029 -r /dev/ttyUSB0 -s 115200 -T localhost -t 4532

# For Yaesu FT-991A
rigctld -m 135 -r /dev/ttyUSB0 -s 38400 -T localhost -t 4532

# For Kenwood TS-480
rigctld -m 305 -r /dev/ttyUSB0 -s 4800 -T localhost -t 4532

# For Elecraft K3
rigctld -m 2041 -r /dev/ttyUSB0 -s 38400 -T localhost -t 4532
```

Common rig model numbers:
- `1024` - Icom IC-7300
- `1029` - Icom IC-705
- `135` - Yaesu FT-991A
- `305` - Kenwood TS-480
- `2041` - Elecraft K3

See `rigctl --list` for full list of supported rigs.

### Connecting in QSOLink

1. Start rigctld with your radio connected
2. Click "Rig Settings" in the toolbar
3. Enter host (default: localhost) and port (default: 4532)
4. Click "Connect"

When connected:
- The header shows a green indicator with current frequency
- Frequency and mode auto-populate in the contact form
- The indicator turns red when disconnected

## Log Export

### ADIF Export

1. Click "Export ADIF" in the toolbar
2. File is saved as `qso_YYYYMMDD.adif`

### Cabrillo Export

1. Click "Export Cabrillo" in the toolbar
2. File is saved as `qso_YYYYMMDD.cabrillo`

## Project Structure

```
QSOLink-client/
├── src/
│   ├── app.rs         # Main UI application
│   ├── db.rs          # SQLite database operations
│   ├── export.rs      # ADIF/Cabrillo export
│   ├── lotw.rs        # ARRL LoTW API client and ADIF parser
│   ├── models.rs      # Data models, validation, ADIF mode info
│   ├── qrz.rs         # QRZ.com API client
│   ├── remote_db.rs   # Remote database support
│   ├── rigctl.rs      # Hamlib rigctld client
│   ├── security.rs    # Credential encryption, station profile persistence
│   └── main.rs        # Entry point
├── Cargo.toml         # Dependencies
└── Cargo.lock         # Locked dependencies
```

## Security

- QRZ.com credentials are encrypted with AES-256-GCM
- SQL injection protection via parameterized queries
- Input validation on all user fields

## License

MIT License - See LICENSE file

## TODO / Coming Soon

### Transceiver Integration (CAT/CIV) - DONE
- Read frequency and mode directly from radio via Hamlib rigctld
- Auto-populate frequency and mode in contact form
- Status indicator in header (green = connected, red = disconnected)

### Logbook of the World (LoTW) - IN PROGRESS
- [x] ARRL LoTW integration (check confirmations via lotw.arrl.org)
- [x] Mark contacts as lotw_submitted / lotw_confirmed
- [x] Periodic sync on startup and hourly while running
- [x] Station profile setup at first launch
- [x] Mode dropdown with ADIF-enumerated values
- [x] Maidenhead grid square validation
- [x] ADIF export with STATION_CALLSIGN, MY_GRIDSQUARE, SUBMODE for LoTW
- [ ] **TQSL direct integration** (sign and upload ADIF via TrustedQSL) — deferred to future release

### Additional Features
- Contact editing (update existing contacts)
- Duplicate contact detection
- Statistics dashboard (QSOs per band/mode)
- Multi-user support for remote database
- Backup/restore functionality
- Custom field support
