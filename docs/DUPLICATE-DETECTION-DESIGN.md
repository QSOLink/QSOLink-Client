# Design Document: Duplicate Contact Detection

## Overview

Detect and warn users when attempting to log a contact that may already exist. Duplicate detection prevents data clutter and ensures accurate statistics.

---

## What Counts as a Duplicate?

### Primary Key (Must Match)

| Field | Rationale |
|-------|-----------|
| Callsign | Core identifier |
| Date | Same day contact |
| Time On | Within same QSO |

### Secondary Indicators (Support Decision)

| Field | Impact |
|-------|--------|
| Band | Same station, same band = high confidence duplicate |
| Mode | Same mode increases confidence |
| Frequency | Exact match = very high confidence |

---

## Duplicate Detection Strategies

### Strategy 1: Exact Match (Strict)

Same callsign + date + time = duplicate

**Pros:** Simple, predictable
**Cons:** May miss typos (N6ABC vs N8ABC)

### Strategy 2: Fuzzy Match (Permissive)

Same callsign + same day = possible duplicate

**Pros:** Catches more issues
**Cons:** False positives for repeat QSOs with same station

### Strategy 3: Configurable Threshold (Recommended)

| Match Level | Criteria | Action |
|-------------|----------|--------|
| Exact | CALLSIGN + DATE + TIME + BAND | Block with confirmation |
| Strong | CALLSIGN + DATE + TIME | Warn, require confirmation |
| Possible | CALLSIGN + DATE + BAND | Warn, no block |
| Weak | CALLSIGN + DATE | Suggest check |

---

## User Experience

### On Duplicate Detection

```
┌─────────────────────────────────────────────────────────┐
│ ⚠ Possible Duplicate Contact                        [X]│
├─────────────────────────────────────────────────────────┤
│ This contact matches an existing record:                │
│                                                         │
│   Callsign:  N6ABC                                      │
│   Date:      2026-03-20                                │
│   Time:      14:30                                     │
│   Band:      20m                                       │
│   Mode:      SSB                                       │
│                                                         │
│ Existing notes: " contest exchange 599 CA"              │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ [Add Anyway]  [Update Existing]  [Cancel]              │
└─────────────────────────────────────────────────────────┘
```

### Options

| Button | Behavior |
|--------|----------|
| **Add Anyway** | Force add the contact |
| **Update Existing** | Open edit modal for existing contact |
| **Cancel** | Discard new entry |

---

## Data Model

### New Fields on Contact

| Field | Type | Description |
|-------|------|-------------|
| `duplicate_of` | uuid | Reference to original if this is a duplicate |
| `is_duplicate` | boolean | Flag indicating this is a duplicate entry |

### New Table: Duplicate Log

| Field | Type | Description |
|-------|------|-------------|
| `id` | uuid | Primary key |
| `new_contact_id` | uuid | The newly logged contact |
| `existing_contact_id` | uuid | The matched existing contact |
| `match_type` | text | "exact", "strong", "possible", "weak" |
| `user_action` | text | "added", "ignored", "updated" |
| `created_at` | timestamp | When detection occurred |

---

## Implementation Details

### Detection Algorithm

```rust
fn find_duplicates(contact: &NewContact, db: &Database) -> Vec<DuplicateMatch> {
    let mut matches = Vec::new();

    // Exact match
    if let Some(exact) = db.find_contact(
        call: contact.callsign,
        date: contact.date,
        time: contact.time_on,
        band: contact.band
    ) {
        matches.push(DuplicateMatch {
            contact: exact,
            level: MatchLevel::Exact,
        });
    }

    // Strong match (no band)
    if let Some(strong) = db.find_contact(
        call: contact.callsign,
        date: contact.date,
        time: contact.time_on,
    ) {
        if matches.is_empty() {
            matches.push(DuplicateMatch {
                contact: strong,
                level: MatchLevel::Strong,
            });
        }
    }

    // Possible match (no time)
    let possible = db.find_contacts(
        call: contact.callsign,
        date: contact.date,
    );

    for p in possible {
        if matches.iter().all(|m| m.contact.id != p.id) {
            matches.push(DuplicateMatch {
                contact: p,
                level: MatchLevel::Possible,
            });
        }
    }

    matches
}
```

---

## Configuration Options

```toml
[duplicate_detection]
enabled = true
strict_mode = false          # true = block exact matches

[duplicate_detection.thresholds]
exact_band = true            # Block exact matches
strong = true                # Warn on strong matches
possible = false             # Don't warn on possible matches

[duplicate_detection.time_tolerance_minutes]
# Consider times within N minutes as the same QSO
default = 5
```

---

## Sync Implications

### With Multi-Node Sync

- Duplicate detection runs on each device
- Same duplicate may be flagged on multiple nodes
- User may resolve differently on each device
- Server reconciliation handles divergence

---

## Edge Cases

| Case | Handling |
|------|----------|
| Duplicate across devices | Each device detects independently; merge handles it |
| Duplicate of a duplicate | Follow the chain |
| User ignores warning | Log that user ignored; don't flag again |
| Contest exchange differs | "Update Existing" updates notes field only |
| Time zone issues | Always store in UTC; compare UTC times |

---

## Implementation Steps

- [ ] Add database migration for duplicate fields
- [ ] Create duplicate_matches table
- [ ] Implement detection algorithm in `db.rs`
- [ ] Add configuration options
- [ ] Create duplicate warning modal UI
- [ ] Handle "Add Anyway" flow
- [ ] Handle "Update Existing" flow
- [ ] Add settings toggle in preferences
- [ ] Log duplicate detection events
- [ ] Add unit tests with edge cases
- [ ] Update sync to propagate duplicate relationships

---

## Open Questions

1. Should duplicate detection run on import (ADIF)?
2. Should QSOs from contests with rapid-fire contacts be excluded?
3. Should the duplicate threshold be configurable per-band or global?
4. Should we auto-update existing contact notes instead of warning?
