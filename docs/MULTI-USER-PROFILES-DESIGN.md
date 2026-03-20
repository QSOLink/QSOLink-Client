# Design Document: Multi-User Support (Operator Profiles)

## Overview

Support multiple operators on a shared device or database. Each operator has their own profile and logbook, with the ability to switch between profiles at runtime.

---

## Core Concept: Operator Profiles

An **Operator Profile** contains:
- Personal information (callsign, name, license class)
- Station information (station callsign, grid square, QTH)
- Preferences (theme, default band/mode)
- Associated logbook (contacts created by this operator)

A **Station** is the physical location/radio setup. Multiple operators can share a station.

---

## Data Model

### Operator Table

```sql
CREATE TABLE operators (
    id TEXT PRIMARY KEY,          -- UUID
    callsign TEXT UNIQUE NOT NULL,
    first_name TEXT,
    last_name TEXT,
    license_class TEXT,           -- " technician", "general", "extra"
    email TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Station Table

```sql
CREATE TABLE stations (
    id TEXT PRIMARY KEY,
    operator_id TEXT REFERENCES operators(id),
    name TEXT,                    -- "Home", "Field", "DXpedition"
    callsign TEXT,                -- Station callsign (may differ from operator)
    grid_square TEXT,
    address TEXT,
    latitude REAL,
    longitude REAL,
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Contact Changes

```sql
-- Add to existing contacts table
ALTER TABLE contacts ADD COLUMN operator_id TEXT REFERENCES operators(id);
ALTER TABLE contacts ADD COLUMN station_id TEXT REFERENCES stations(id);
```

---

## UI Design

### Profile Selector (Header Bar)

```
┌─────────────────────────────────────────────────────────────────────────┐
│ [Logo] QSOLink          [📡 N6ABC ▼]  [⚙ Settings]  [❓ Help]       │
│                            ↑ Click to switch profile                   │
└─────────────────────────────────────────────────────────────────────────┘
```

### Profile Switcher Dropdown

```
┌─────────────────────────────────────┐
│  👤 N6ABC (You)                ← current
│  👤 W1ABC (Bill)
│  👤 K2XYZ (Sarah)
│  ──────────────────────────────────
│  ➕ Add New Operator
│  ⚙ Manage Profiles
└─────────────────────────────────────┘
```

### Profile Management Screen

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Operator Profiles                                          [X]       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─ Active: N6ABC ─────────────────────────────────────────────────┐  │
│  │                                                                      │  │
│  │  Callsign:     [N6ABC                    ]                       │  │
│  │  First Name:   [John                     ]                       │  │
│  │  Last Name:    [Smith                    ]                       │  │
│  │  License:      [Extra ▼                  ]                       │  │
│  │  Email:        [john@n6abc.com           ]                       │  │
│  │                                                                      │  │
│  │  Station:                                                           │  │
│  │  ┌────────────────────────────────────────────────────────────┐   │  │
│  │  │  Name:       [Home Station]                                │   │  │
│  │  │  Callsign:   [N6ABC            ]                          │   │  │
│  │  │  Grid:       [DM04             ]                          │   │  │
│  │  │  QTH:        [San Francisco, CA]                         │   │  │
│  │  │  Lat/Lon:    [37.7749, -122.4194]                         │   │  │
│  │  │  [✓] Default station                                      │   │  │
│  │  └────────────────────────────────────────────────────────────┘   │  │
│  │                                                                      │  │
│  │  [+ Add Station]                                                   │  │
│  │                                                                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌─ Other Operators ───────────────────────────────────────────────┐  │
│  │                                                                      │  │
│  │  👤 W1ABC (Bill)           [Edit]  [Delete]  [Set Active]       │  │
│  │  👤 K2XYZ (Sarah)          [Edit]  [Delete]  [Set Active]       │  │
│  │                                                                      │  │
│  │  [+ Add Operator]                                                   │  │
│  │                                                                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│                              [Cancel]  [Save Changes]                    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Profile Switching Behavior

| Scenario | Behavior |
|----------|----------|
| App startup | Load last active operator's logbook |
| Switch operator | Prompt to save unsaved contacts, then load that operator's logbook |
| Shared database | Each operator sees only their contacts |
| Local SQLite | Create separate database per operator OR filter by operator_id |

---

## Logbook Filtering

When an operator is active, the logbook shows only their contacts:

```rust
fn get_contacts(operator_id: &str, db: &Database) -> Vec<Contact> {
    db.query(
        "SELECT * FROM contacts WHERE operator_id = ? ORDER BY date DESC, time_on DESC",
        [operator_id]
    )
}
```

---

## Station Profile Usage

When logging a new QSO:
1. Pre-fill station callsign and grid from active station
2. User can override if operating from different location
3. Contact is tagged with both operator_id and station_id

---

## First Launch / New User Flow

```
┌─────────────────────────────────────────────────────────┐
│  Welcome to QSOLink!                                   │
│                                                         │
│  Let's set up your operator profile.                   │
│                                                         │
│  Your Callsign: [_________]                           │
│                                                         │
│  [Get Started]                                         │
└─────────────────────────────────────────────────────────┘
```

---

## Shared Database Scenario

For club stations or shared remote database:

| Mode | Description |
|------|-------------|
| **Personal** | Each operator has own SQLite file; no sharing |
| **Shared (Remote)** | All operators on same PostgreSQL/MySQL; filtered by operator_id |
| **Club** | All contacts visible, operator_id tracked on each contact |

---

## Sync Implications

With multi-node sync enabled:

| Challenge | Solution |
|-----------|----------|
| Same operator on multiple devices | Sync by operator_id |
| Different operators on same device | Each profile can sync independently |
| Conflicts | Operator scope prevents most conflicts (different operators = different data) |

---

## Implementation Steps

- [ ] Create operators table migration
- [ ] Create stations table migration
- [ ] Add operator_id/station_id to contacts
- [ ] Build operator selector dropdown in header
- [ ] Build profile management screen
- [ ] Build station management (within profile)
- [ ] Implement profile switching logic
- [ ] Filter logbook by active operator
- [ ] Pre-fill station info on new QSO
- [ ] First-launch profile creation wizard
- [ ] Persist last active operator
- [ ] Add profile indicator to exported ADIF (STATION_CALLSIGN)

---

## Open Questions

1. Should operators be able to see each other's contacts in shared database mode?
2. Should there be an "admin" role that can manage all profiles?
3. How to handle QSO counter (e.g., "QSO #1234") — per operator or per station?
4. Should profile switching require confirmation if there are unsaved changes?
5. How to handle operators who share the same callsign (e.g., family members)?
