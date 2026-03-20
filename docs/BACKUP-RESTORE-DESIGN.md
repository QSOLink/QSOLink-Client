# Design Document: Backup and Restore

## Overview

Provide reliable backup and restore functionality to protect user data from loss. Includes automated backups, manual exports, and disaster recovery.

---

## Backup Types

| Type | Description | Use Case |
|------|-------------|----------|
| **Full Backup** | Complete database export (SQL or JSON) | Disaster recovery |
| **ADIF Export** | QSO data in standard ADIF format | Import to other software |
| **Settings Backup** | App configuration and credentials | Migrate to new device |
| **Incremental Backup** | Only new contacts since last backup | Regular automation |

---

## Export Formats

### ADIF (Recommended for Users)

Standard amateur radio format compatible with:
- LoTW (with proper fields)
- eQSL
- Club Log
- Other logging software

```adi
<ADIF_VER:5>3.1.4
<PROGRAMID:7>QSOLink
<EOH>

<CALL:5>N6ABC
<QSO_DATE:8>20260320
<TIME_ON:4>1430
<BAND:3>20m
<MODE:3>SSB
<RST_SENT:2>59
<RST_RCVD:2>59
<NAME:4>John
<QTH:13>San Francisco
<GRIDSQUARE:4>DM04
<EOR>

<CALL:5>VK3ABC
...
```

### JSON Export (Recommended for Migration)

Full fidelity backup including:
- All contact fields
- LoTW confirmation status
- Custom fields
- Settings

```json
{
  "version": "1.0",
  "exported_at": "2026-03-20T14:30:00Z",
  "contacts": [
    {
      "id": "uuid",
      "callsign": "N6ABC",
      "date": "2026-03-20",
      "time_on": "14:30",
      "band": "20m",
      "mode": "SSB",
      ...
    }
  ],
  "settings": {...},
  "operators": [...]
}
```

### SQL Dump

For advanced users and database migration:
```bash
pg_dump -Fc qsolink > backup.dump
mysql dump qsolink > backup.sql
```

---

## UI Design

### Backup Screen

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Backup & Restore                                                [X]  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─ Quick Backup ──────────────────────────────────────────────────┐  │
│  │                                                                      │  │
│  │  [📥 Export ADIF]    [📥 Export JSON]    [📥 Full Backup]        │  │
│  │                                                                      │  │
│  │  Last backup: 2026-03-19 14:30 (ADIF)                             │  │
│  │                                                                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌─ Automated Backups ──────────────────────────────────────────────┐  │
│  │                                                                      │  │
│  │  [✓] Enable automatic backups                                       │  │
│  │  Schedule: [Daily ▼]  Time: [02:00 ▼]                             │  │
│  │  Keep last: [7 ▼] backups                                          │  │
│  │  Location: [~/QSOLink/backups/     ]  [Browse...]                 │  │
│  │                                                                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌─ Restore ───────────────────────────────────────────────────────┐  │
│  │                                                                      │  │
│  │  [📤 Select Backup File...]                                       │  │
│  │                                                                      │  │
│  │  ⚠ Warning: Restoring will merge with existing contacts.           │  │
│  │     Duplicates will be skipped.                                     │  │
│  │                                                                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌─ Backup History ────────────────────────────────────────────────┐  │
│  │                                                                      │  │
│  │  2026-03-19 14:30  backup_full.json      2.3 MB  [Restore]     │  │
│  │  2026-03-18 02:00  backup_incremental    45 KB   [Restore]     │  │
│  │  2026-03-17 02:00  backup_full.json      2.3 MB  [Restore]     │  │
│  │                                                                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Automated Backup Strategy

### Local Storage

```toml
[backup]
enabled = true
schedule = "daily"
time = "02:00"
keep = 7  # Number of backups to retain
path = "~/.qsolink/backups"
```

### Cloud Storage (Future)

| Provider | Implementation |
|----------|----------------|
| Dropbox | OAuth2 API |
| Google Drive | OAuth2 API |
| S3 | AWS SDK |
| WebDAV | Standard protocol |

---

## Restore Flow

```
┌─────────────────────────────────────────────────────────┐
│  Restore from Backup                                     │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Selected: backup_full_20260319.json                     │
│  Contacts: 1,247                                         │
│  Date range: 2024-01-01 to 2026-03-19                   │
│                                                          │
│  ┌─ Restore Options ──────────────────────────────────┐ │
│  │                                                           │ │
│  │  (•) Merge with existing contacts                      │ │
│  │      Duplicates will be skipped                         │ │
│  │                                                           │ │
│  │  ( ) Replace all contacts                               │ │
│  │      ⚠ This will delete all existing contacts           │ │
│  │                                                           │ │
│  │  [ ] Restore settings too                               │ │
│  │                                                           │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                          │
│  [Cancel]                    [Restore]                    │
└─────────────────────────────────────────────────────────┘
```

---

## Import Handling

### Duplicate Detection on Import

| Rule | Action |
|------|--------|
| Same CALLSIGN + DATE + TIME | Skip (duplicate) |
| Same CALLSIGN + DATE + different TIME | Prompt user |
| New contact | Import |

### Import Results

```
┌─────────────────────────────────────────────────────────┐
│  Import Complete                                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ✓ 1,195 contacts imported                              │
│  ✓ 52 duplicates skipped                                │
│  ✗ 3 contacts failed (validation errors)               │
│                                                          │
│  [Show Failed Contacts]                                  │
│                                                          │
│  [Close]                                                │
└─────────────────────────────────────────────────────────┘
```

---

## Implementation Details

### File Naming Convention

```
backup_full_YYYYMMDD_HHMMSS.json
backup_incremental_YYYYMMDD_HHMMSS.json
```

### Backup Metadata

```json
{
  "version": "1.0",
  "qsolink_version": "0.3.0",
  "exported_at": "2026-03-20T14:30:00Z",
  "backup_type": "full",
  "checksum": "sha256:abc123...",
  "contact_count": 1247,
  "date_range": {
    "oldest": "2024-01-15",
    "newest": "2026-03-19"
  }
}
```

---

## Implementation Steps

- [ ] Create backup module (`backup.rs`)
- [ ] Implement ADIF export with all fields
- [ ] Implement JSON export with full fidelity
- [ ] Create backup settings UI
- [ ] Implement automated backup scheduler
- [ ] Add backup rotation (keep N backups)
- [ ] Implement restore flow with merge/replace options
- [ ] Add duplicate detection on import
- [ ] Create import results UI
- [ ] Add backup integrity verification (checksum)
- [ ] Implement settings export/import
- [ ] Add backup to cloud storage (future)
- [ ] Document recovery procedure

---

## Open Questions

1. Should backups be encrypted by default?
2. Should we support incremental ADIF exports (only new QSOs)?
3. How to handle custom fields in ADIF export (ADIF doesn't support natively)?
4. Should restore create a backup of current state first?
5. What is the maximum backup file size to support?
