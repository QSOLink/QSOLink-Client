# Design Document: Contact Editing

## Overview

Allow users to modify existing QSO records. Contacts are immutable once synced in multi-node scenarios, so editing requires conflict-aware strategies.

---

## Use Cases

| UC | Scenario | Behavior |
|----|----------|----------|
| 1 | Typo fix | Correct misspelled callsign or name |
| 2 | Missing data | Add missing RST, QTH, or notes |
| 3 | Wrong data | Correct incorrect band, mode, or frequency |
| 4 | Time correction | Fix logged time if operator error |
| 5 | Delete contact | Remove accidentally logged contact |

---

## Data Model Changes

### New Fields

| Field | Type | Description |
|-------|------|-------------|
| `updated_at` | timestamp | Last modification time |
| `updated_by` | text | Device ID that made last edit |
| `is_deleted` | boolean | Soft delete flag |

### Schema Migration

```sql
ALTER TABLE contacts ADD COLUMN updated_at TIMESTAMP DEFAULT NULL;
ALTER TABLE contacts ADD COLUMN updated_by TEXT DEFAULT NULL;
ALTER TABLE contacts ADD COLUMN is_deleted BOOLEAN DEFAULT FALSE;
```

---

## UI Design

### Entry Points

1. **Double-click** contact row in logbook view → opens edit modal
2. **Right-click** contact → context menu with "Edit" option
3. **Select + Edit button** in toolbar

### Edit Modal

```
┌─────────────────────────────────────────────────────┐
│ Edit Contact                                   [X]  │
├─────────────────────────────────────────────────────┤
│ Callsign: [N6ABC          ]                         │
│ Date:     [2026-03-20     ]                        │
│ Time On:  [14:30         ]  Time Off: [15:45    ] │
│ Band:     [20m ▼         ]  Mode: [SSB ▼         ] │
│ Frequency:[14.250         ]  RST Sent: [59 ▼     ] │
│ RST Rec:  [59  ▼         ]                         │
│ Name:     [John           ]                        │
│ QTH:      [Los Angeles    ]                        │
│ Grid:     [DM04           ]                        │
│ Notes:    [___________________________]            │
├─────────────────────────────────────────────────────┤
│                              [Cancel]  [Save]      │
└─────────────────────────────────────────────────────┘
```

---

## Validation Rules

| Field | Rule |
|-------|------|
| Callsign | Required, valid format, uppercase |
| Date | Required, not in future |
| Time On | Required, valid HH:MM format |
| Time Off | Optional, must be >= Time On |
| Band | Required, from standard band list |
| Mode | Required, from ADIF mode list |
| Frequency | Optional, must match band |
| RST | Required for HF, optional for VHF+ |
| Grid | Optional, valid 4 or 6 character Maidenhead |

---

## Sync Implications

### Phase 1 (Simple Sync)

- Edits are overwritten by server on next sync (last-write-wins)
- `updated_at` and `updated_by` are informational only

### Phase 2+ (CRDT Sync)

- Edits are immutable events (edit = delete + create new)
- Server merges edits from all nodes
- Conflicts auto-resolve or flag for user

---

## Edge Cases

| Case | Handling |
|------|----------|
| Edit synced contact offline | Allow; flag as "pending sync" |
| Same contact edited on 2 devices | Later timestamp wins (Phase 1) |
| Delete synced contact | Soft delete; propagates to all nodes |
| Edit LoTW-confirmed contact | Allow; note original submission |

---

## Implementation Steps

- [ ] Add database migration for new columns
- [ ] Update `models.rs` with new fields
- [ ] Add edit button and context menu to logbook UI
- [ ] Create edit modal component
- [ ] Implement validation logic
- [ ] Update `db.rs` with update methods
- [ ] Add `updated_at`/`updated_by` on save
- [ ] Handle soft delete
- [ ] Add unit tests for validation
- [ ] Update sync to handle edit propagation

---

## Open Questions

1. Should edits be logged in an audit trail?
2. Can users edit contacts that are LoTW confirmed?
3. Should there be a limit on how long after logging a contact can be edited?
