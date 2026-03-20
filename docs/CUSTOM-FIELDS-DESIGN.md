# Design Document: Custom Fields

## Overview

Allow users to define custom data fields beyond the standard QSO fields. Useful for contest exchanges, special information, or personal tracking.

---

## Use Cases

| Use Case | Custom Fields |
|----------|--------------|
| **Contest Exchange** | Serial number, points, exchange code |
| **Satellite QSO** | Satellite name, mode, antenna, power |
| **HF Maritime** | Vessel name, MMSI, vessel type |
| **Emergency/Public Service** | Operator role, net name, check-in time |
| **Awards Tracking** | Reference number, award ID |
| **Personal Notes** | Any user-defined data |

---

## Data Model

### Custom Field Definition

```sql
CREATE TABLE custom_fields (
    id TEXT PRIMARY KEY,
    operator_id TEXT REFERENCES operators(id),
    name TEXT NOT NULL,
    field_type TEXT NOT NULL,  -- 'text', 'number', 'select', 'checkbox', 'date'
    options TEXT,              -- JSON array for 'select' type
    required BOOLEAN DEFAULT FALSE,
    show_on_list BOOLEAN DEFAULT FALSE,  -- Show in logbook list view
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Custom Field Values

```sql
CREATE TABLE custom_field_values (
    id TEXT PRIMARY KEY,
    contact_id TEXT REFERENCES contacts(id),
    field_id TEXT REFERENCES custom_fields(id),
    value TEXT,  -- JSON-encoded value
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(contact_id, field_id)
);
```

### Field Types

| Type | Storage | UI Component |
|------|---------|--------------|
| `text` | String | Single-line input |
| `textarea` | String | Multi-line input |
| `number` | Number | Number input with optional min/max |
| `select` | String | Dropdown from options list |
| `checkbox` | Boolean | Toggle |
| `date` | Date | Date picker |
| `time` | Time | Time picker |
| `callsign` | String | Callsign input with validation |

---

## Preset Templates

Provide common templates users can start with:

### Contest Exchange

```json
{
  "name": "Contest Exchange",
  "fields": [
    { "name": "Exchange", "type": "text", "required": true },
    { "name": "Serial Sent", "type": "number" },
    { "name": "Serial Received", "type": "number" },
    { "name": "Points", "type": "number" }
  ]
}
```

### Satellite QSO

```json
{
  "name": "Satellite",
  "fields": [
    { "name": "Satellite Name", "type": "select", "options": ["AO-91", "AO-92", "SO-50", "..."] },
    { "name": "Satellite Mode", "type": "select", "options": ["FM", "SSB", "CW", "FT8"] },
    { "name": "Antenna", "type": "text" },
    { "name": "Power", "type": "text" }
  ]
}
```

### Maritime Mobile

```json
{
  "name": "Maritime Mobile",
  "fields": [
    { "name": "Vessel Name", "type": "text" },
    { "name": "MMSI", "type": "number" },
    { "name": "Vessel Type", "type": "select", "options": ["Cargo", "Tanker", "Sailing", "Fishing", "..."] },
    { "name": "IMO Number", "type": "text" }
  ]
}
```

---

## UI Design

### Custom Fields Settings

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Custom Fields                                                [X]     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─ Manage Fields ────────────────────────────────────────────────┐  │
│  │                                                                      │  │
│  │  [+ Add Field]  [📋 Load Template]                                │  │
│  │                                                                      │  │
│  │  ┌──────────────────────────────────────────────────────────────┐  │  │
│  │  │ ≡  Satellite Name      [select ▼]      [✓] Required  [⋮]    │  │  │
│  │  │     AO-91, AO-92, SO-50...                                  │  │  │
│  │  └──────────────────────────────────────────────────────────────┘  │  │
│  │  ┌──────────────────────────────────────────────────────────────┐  │  │
│  │  │ ≡  Antenna             [text ▼]        [ ] Required  [⋮]    │  │  │
│  │  └──────────────────────────────────────────────────────────────┘  │  │
│  │  ┌──────────────────────────────────────────────────────────────┐  │  │
│  │  │ ≡  Power              [text ▼]        [ ] Required  [⋮]    │  │  │
│  │  └──────────────────────────────────────────────────────────────┘  │  │
│  │                                                                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌─ Add New Field ─────────────────────────────────────────────────┐  │
│  │                                                                      │  │
│  │  Field Name:  [________________]                                 │  │
│  │  Field Type:  [select ▼]                                          │  │
│  │  Options:     [________________]  (comma-separated)               │  │
│  │  [✓] Required                                                   │  │
│  │  [✓] Show in logbook list                                        │  │
│  │                                                                      │  │
│  │                           [Cancel]  [Add Field]                   │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Template Selector

```
┌─────────────────────────────────────────────────────────┐
│  Load Template                                      [X] │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  [🔍 Search templates...]                               │
│                                                          │
│  ┌─────────────────────────────────────────────────────┐ │
│  │  📡 Contest Exchange                                │ │
│  │     Serial number, points, exchange code            │ │
│  └─────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────┐ │
│  │  🛰️ Satellite QSO                                  │ │
│  │     Satellite name, mode, antenna, power            │ │
│  └─────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────┐ │
│  │  🚢 Maritime Mobile                                 │ │
│  │     Vessel name, MMSI, vessel type                  │ │
│  └─────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────┐ │
│  │  📞 Emergency/Public Service                        │ │
│  │     Role, net name, check-in time                   │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                          │
│  [Cancel]              [Load Selected Template]           │
└─────────────────────────────────────────────────────────┘
```

### Contact Form with Custom Fields

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Log Contact                                                       [X] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Callsign:  [N6ABC           ]                                        │
│  Date:      [2026-03-20     ]  Time: [14:30          ]               │
│  Band:      [20m ▼          ]  Mode: [SSB ▼         ]               │
│  RST Sent:  [59              ]  RST Rec: [59         ]               │
│  Name:      [John           ]                                        │
│  QTH:       [Los Angeles    ]                                        │
│  Grid:      [DM04           ]                                        │
│                                                                         │
│  ── Custom Fields ──────────────────────────────────────────────────── │
│                                                                         │
│  Satellite Name: [AO-91 ▼           ]                                │
│  Antenna:        [Cross Yagi ▼       ]                                │
│  Power:          [10W                ]                                │
│                                                                         │
│  Notes:      [_____________________________________________________]  │
│                                                                     [__] │
│                                                                         │
│                                      [Cancel]  [Log Contact]           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Logbook List View

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│  QSO Log                    [🔍 Search...]  [⚙ Columns]  [📥 Export]          │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Date       Time   Call      Name     QTH        Band  Mode  Sat Name  ▲  │
│  ────────────────────────────────────────────────────────────────────────────│
│  03/20    14:30   N6ABC     John     CA          20m   SSB   AO-91    ▼  │
│  03/19    10:15   VK3ABC    Mike     Australia   15m   FT8   SO-50    │  │
│  03/18    18:42   G4XYZ     Bob      England     40m   CW    —        │  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## ADIF Export Considerations

ADIF 3.1.4 supports user-defined fields via `<USERDEF>`. Custom fields can be exported as:

```adi
<USERDEF:12>Satellite Name:1:1
<USERDEF:9>Antenna:1:2
<USERDEF:5>Power:1:3
```

Or mapped to existing ADIF fields where applicable.

---

## Sync Considerations

With multi-user/sync enabled:

- Custom field definitions sync with operator profile
- Custom field values sync with contacts
- New field added on one device propagates to others
- Contacts without a field have NULL value

---

## Implementation Steps

- [ ] Create custom_fields table migration
- [ ] Create custom_field_values table migration
- [ ] Build custom field definition UI
- [ ] Build template selector UI
- [ ] Implement template loading logic
- [ ] Add custom fields to contact form
- [ ] Implement field type renderers
- [ ] Add validation for required fields
- [ ] Add custom fields to logbook list view
- [ ] Implement ADIF export for custom fields
- [ ] Implement JSON export for custom fields
- [ ] Add import handling for custom field data
- [ ] Sync custom field definitions with operator profile
- [ ] Document USERDEF format for ADIF

---

## Open Questions

1. Should custom fields be per-operator or per-station?
2. Should we limit the number of custom fields?
3. How to handle custom fields in statistics/awards?
4. Should custom fields be searchable in the main search?
5. How to handle field name conflicts between operators in shared database?
