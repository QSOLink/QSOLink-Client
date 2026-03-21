# FCC ULS Database Integration

## Title

As a US amateur radio operator, I want QSOLink to use the FCC's Universal Licensing System (ULS) database as the primary source of truth for US callsign lookups so I can get accurate licensee information without depending on third-party services.

---

## Story

**As a** US amateur radio operator who frequently works DX and contacts other US stations,

**I want** QSOLink to automatically lookup US callsigns using the official FCC database,

**So that** I get accurate name, location, and license class information without requiring an internet connection or third-party subscription.

---

## Background & Motivation

- QRZ.com provides excellent callsign lookup but requires a paid subscription and has rate limits
- The FCC publishes the complete Amateur Radio Service database weekly as free downloads
- Delta files are available daily for incremental updates
- A local FCC ULS copy provides:
  - Fast, offline-capable lookups
  - No rate limiting or subscription requirements
  - Official government source of truth
  - Complete coverage of all US amateur licenses

---

## User Configuration Options

### Settings Panel

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `fcc_uls_enabled` | bool | `false` | Enable/disable FCC ULS integration |
| `fcc_uls_path` | path | `~/.config/qsolog/fcc_uls.db` | Local SQLite database path |
| `fcc_full_sync_day` | enum | `Sunday` | Day of week for full database pull |
| `fcc_full_sync_hour` | u8 | `2` | Hour (UTC) to run weekly full sync |
| `fcc_delta_sync_enabled` | bool | `true` | Enable daily delta updates |
| `fcc_delta_sync_hour` | u8 | `3` | Hour (UTC) to run daily delta sync |

---

## Data Synchronization Strategy

### Weekly Full Pull

- **Schedule**: Configurable day of week, default Sunday at 02:00 UTC
- **Source**: https://www.fcc.gov/uls/transactions/daily-weekly
- **File**: `l_amat.zip` — Amateur Radio Service database
- **Process**:
  1. Download latest weekly build
  2. Extract and parse LUI (License Update) files
  3. Replace existing local database (or use temp table swap for atomicity)
  4. Create/rebuild indexes for optimal query performance
  5. Update metadata table with sync timestamp

### Daily Delta Pull

- **Schedule**: Configurable hour, default 03:00 UTC
- **Source**: Same FCC ULS daily files
- **Process**:
  1. Check if daily delta file exists and is newer than last delta applied
  2. Download delta file if available
  3. Apply changes to local database (INSERT/UPDATE/DELETE based on action codes)
  4. Update metadata table with delta timestamp
  5. If delta is unavailable (weekend/holiday), skip silently

### Data Retention

- Keep delta files applied to database (not stored separately)
- Store metadata: last_full_sync, last_delta_sync, database_version
- Full rebuild automatically if delta gap exceeds 7 days

---

## Database Schema

### Tables

```sql
-- Metadata for sync tracking
CREATE TABLE fcc_uls_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Header table (call sign -> entity mapping)
CREATE TABLE fcc_hd (
    uls_file_number TEXT,
    ebfn_number TEXT,
    call_sign TEXT PRIMARY KEY,
    license_status TEXT,
    grant_date TEXT,
    expired_date TEXT,
    cancellation_date TEXT,
    entity_type TEXT,
    entity_id INTEGER
);

-- Entity table (names, addresses, locations)
CREATE TABLE fcc_en (
    entity_id INTEGER PRIMARY KEY,
    entity_name TEXT,
    first_name TEXT,
    middle_name TEXT,
    last_name TEXT,
    suffix TEXT,
    street_address_1 TEXT,
    street_address_2 TEXT,
    city TEXT,
    state TEXT,
    zip_code TEXT,
    country TEXT
);

-- Amateur-specific data (license class, etc.)
CREATE TABLE fcc_am (
    entity_id INTEGER PRIMARY KEY,
    operator_class TEXT,
    previous_operator_class TEXT,
    statute TEXT
);

-- Indexes for fast lookups
CREATE INDEX idx_hd_call_sign ON fcc_hd(call_sign);
CREATE INDEX idx_hd_status ON fcc_hd(license_status);
CREATE INDEX idx_en_state ON fcc_en(state);
CREATE INDEX idx_en_name ON fcc_en(entity_name);
```

---

## Lookup Integration

### Callsign Lookup Flow

```
1. User enters callsign W1AW in contact form
2. Check if callsign matches US pattern (prefixes: K, N, W, AA-AL, AMA-AZ)
3. If US:
   a. Query local FCC ULS database
   b. If found, populate name, city, state, grid from entity data
   c. If not found or ULS disabled, fallback to QRZ.com
4. If non-US:
   a. Query QRZ.com (or HAMQTH as backup)
```

### Data Population

| FCC Field | Contact Field | Notes |
|-----------|---------------|-------|
| call_sign | call_sign | Primary key |
| entity_name / first_name + last_name | name | Combined full name |
| city + state | city, state | Location fields |
| state | county | State-level only |
| — | grid_square | Not in FCC data; use QRZ or manual |
| operator_class | notes | e.g., "Extra", "General" |

---

## Implementation Notes

### Error Handling

- **Download failure**: Log error, retry next scheduled sync; use stale data
- **Corrupt file**: Skip delta, flag for full rebuild on next sync
- **Database locked**: Retry with backoff, fail gracefully
- **FCC site unavailable**: Continue with existing data, retry daily

### Logging

- Log sync start/end with duration
- Log number of records added/updated/deleted
- Log any errors encountered
- Store in application log file

### User Notifications

- Show sync status in settings panel
- Display "FCC ULS: Last sync X hours ago" indicator
- Notify if sync fails 3+ consecutive times

### Performance Considerations

- Initial download is ~500MB compressed, ~2GB uncompressed
- Delta files are typically <50MB
- Full rebuild takes 5-10 minutes on modern hardware
- Queries should complete in <50ms for single callsign lookups

---

## Dependencies

- SQLite with FTS5 for text search (already using rusqlite)
- reqwest for HTTP downloads (already in dependencies)
- tokio for async file operations (already in dependencies)
- cron or tokio-cron for scheduling

---

## Future Enhancements

- Grid square lookup via FCC coordinate data (if available in records)
- ZIP code to grid square conversion table
- Integration with QRZ subscription for non-FCC fields (photos, biography)
