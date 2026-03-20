# Design Document: ARRL Awards Tracking

## Overview

Track and visualize progress toward popular amateur radio awards including WAS, DXCC, POTA, SOTA, and more. Show confirmed vs. unconfirmed counts, missing entities, and maps.

---

## Supported Awards

### ARRL Awards

| Award | Full Name | Requirement |
|-------|-----------|-------------|
| WAS | Worked All States | 100 states (QSL required) |
| WAC | Worked All Continents | 6 continents (QSL required) |
| DXCC | DX Century Club | 100 entities (LoTW accepted) |
| 5BWAS | 5-Band WAS | WAS on 5 bands |
| Triple Play | Triple Play | WAS + WAC + DXCC |

### Other Popular Awards

| Award | Full Name | Requirement |
|-------|-----------|-------------|
| POTA | Parks On The Air | Activate 100 parks |
| SOTA | Summits On The Air | Activate 100 summits |
| IOTA | Islands On The Air | 100 islands |
| GridDXCC | Grid Square DXCC | 100 Maidenhead grids |

---

## Awards Screen Layout

Reference: Grafana dashboard panels with dark theme, metric cards, and progress rings.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Awards Progress                           [⏱ Last sync: 5m ago]   [X] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐           │
│  │    WAS     │ │   DXCC     │ │   POTA     │ │   SOTA     │           │
│  │            │ │            │ │            │ │            │           │
│  │    32      │ │    89      │ │    12      │ │     3      │           │
│  │   / 50     │ │   /100     │ │   /100     │ │   /100     │           │
│  │  [███░░]   │ │  [███████] │ │  [█░░░░]   │ │  [█░░░░]   │           │
│  │   64%      │ │    89%     │ │    12%     │ │     3%     │           │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘           │
│                                                                         │
│  ┌─ Filter: [All ▾] [All Bands ▾] [All Modes ▾] ─────────────────────┐│
│  └──────────────────────────────────────────────────────────────────────┘│
│                                                                         │
│  ┌─ WAS: Worked All States ────────────────────────────── [ℹ] [⋮] [⧉] ┐│
│  │                                                                       ││
│  │  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ ││
│  │                                                                       ││
│  │  ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ ││
│  │  32/50 states worked (18 needed)                                    ││
│  │                                                                       ││
│  │  Confirmed: 24  │  Worked: 8  │  Missing: 18                        ││
│  │                                                                       ││
│  └──────────────────────────────────────────────────────────────────────┘│
│                                                                         │
│  ┌─ Missing States (18) ──────────────────────── [Show Grid] [Map] ────┐│
│  │                                                                       ││
│  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                    ││
│  │  │ AL  │ │ AK  │ │ AZ  │ │ AR  │ │ DE  │ │ HI  │                    ││
│  │  │ ◐   │ │ ◐   │ │ ◐   │ │ ◐   │ │ ◐   │ │ ◐   │                    ││
│  │  │     │ │     │ │     │ │     │ │     │ │     │                    ││
│  │  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘                    ││
│  │  ...                                                                 ││
│  │                                                                       ││
│  │  ◐ = worked (not confirmed)                                           ││
│  └──────────────────────────────────────────────────────────────────────┘│
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```
┌─────────────────────────────────────────────────────────────────────────┐
│  Awards Progress                                                    [X] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐         │
│  │   WAS   │ │  DXCC   │ │   POTA  │ │  SOTA   │ │  IOTA   │         │
│  │  32/50  │ │  89/100 │ │   12    │ │    3    │ │    8    │         │
│  │ ██████░ │ │ ████████│ │   ██░  │ │   █░   │ │   ██░   │         │
│  │ States  │ │ Entities│ │  Parks  │ │ Summits │ │ Islands │         │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘         │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  WAS - Worked All States (32/50)                                │   │
│  │  ████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │   │
│  │                                                                 │   │
│  │  Work Required: AL, AK, AZ, AR, CA*, CO*, CT*, DE*, FL*, GA*  │   │
│  │  (* = confirmed via LoTW)                                      │   │
│  │                                                                 │   │
│  │  [Show Missing States]  [View Confirmed]  [Map View]          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌──────────────────────────────┐  ┌──────────────────────────────┐  │
│  │     States Grid             │  │     World Map                 │  │
│  │                              │  │                               │  │
│  │  AL ░  AK ░  AZ ░  AR ░    │  │      ● ●                     │  │
│  │  CA ●  CO ●  CT ●  DE ●    │  │    ●   ●  ●                  │  │
│  │  FL ●  GA ●  ...           │  │       ●   ●                   │  │
│  │                              │  │  ● = worked  ○ = needed      │  │
│  │  ● = confirmed  ◐ = worked  │  │                               │  │
│  │  ░ = not worked             │  │                               │  │
│  └──────────────────────────────┘  └──────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Entity Data Sources

### States (USA)

| Source | Reliability |
|--------|-------------|
| QTH field with state detection | Moderate |
| Callsign prefix lookup (N6 = CA) | High |
| LoTW state field | Highest |

**Implementation:** Cross-reference QTH, callsign prefix, and LoTW data.

### DXCC Entities

| Source | Reliability |
|--------|-------------|
| ARRL DXCC entity list | Authoritative |
| Callsign prefix → DXCC mapping | Standard |

**Implementation:** Embed ARRL DXCC entity list; match callsign prefix to entity.

### Parks (POTA)

| Source | Reliability |
|--------|-------------|
| Park reference in QTH/notes | Required |
| ADIF: ARRL_PARK_REF | Standard |

**Implementation:** Parse QTH for park references (e.g., "K-1234").

### Summits (SOTA)

| Source | Reliability |
|--------|-------------|
| Summit reference in QTH/notes | Required |
| ADIF: ARRL_SUMMIT_REF | Standard |

### Islands (IOTA)

| Source | Reliability |
|--------|-------------|
| IOTA reference in QTH/notes | Required |
| ADIF: IOTA | Standard |

---

## Confirmation Sources

| Source | ARRL Acceptance |
|--------|-----------------|
| LoTW | All ARRL awards |
| Paper QSL | WAS, WAC, DXCC |
| eQSL | DXCC (some modes only) |
| Lotw_submitted | Not confirmed yet |

**Implementation:** Use existing `lotw_confirmed` and `qsl_sent`/`qsl_recv` fields.

---

## Data Model

### Award Progress Table

```rust
pub struct AwardProgress {
    pub award_id: String,       // "WAS", "DXCC", "POTA", etc.
    pub entity_id: String,      // "CA", "JA", "K-0001", etc.
    pub worked: bool,
    pub confirmed: bool,
    pub first_worked: NaiveDate,
    pub confirmed_date: Option<NaiveDate>,
    pub band: Option<String>,   // For multi-band awards
    pub mode: Option<String>,   // For multi-mode awards
}
```

### Award Definitions

```rust
pub struct AwardDefinition {
    pub id: String,
    pub name: String,
    pub requirement: i32,        // e.g., 50 for states
    pub entity_type: EntityType, // State, Country, Park, Summit, Island
    pub requires_confirmation: bool,
    pub bands_allowed: Vec<String>,  // Empty = all bands
}
```

---

## Visual Design Reference: Grafana-Inspired

Reference: [Grafana Dashboard Gallery](https://grafana.com/grafana/dashboards/) — dark theme, progress indicators, metric cards.

### Color Palette (Awards-Specific)

| Element | Color | Hex |
|---------|-------|-----|
| Progress bar (complete) | Teal | `#00b8d9` |
| Progress bar (partial) | Amber | `#ffab00` |
| Confirmed entity | Green | `#36b37e` |
| Worked (not confirmed) | Purple | `#9f7aea` |
| Missing/needed | Red-orange | `#ff5630` |
| Award card background | Dark card | `#22252b` |
| Map fill (worked) | Teal 30% opacity | `#00b8d94d` |
| Map fill (confirmed) | Green 30% opacity | `#36b37e4d` |

### Award Stat Card

```
┌─────────────────────────┐
│                         │
│      DXCC               │
│                         │
│       89                │
│      /100               │
│                         │
│  ████████████░░░░░░░░░  │
│                         │
│      89%                │
│   11 needed             │
│                         │
└─────────────────────────┘
```

### Progress Bar Style

```
████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
  worked         confirmed          missing
  (teal)         (green)            (amber)
```

### Entity Status Indicator

| Status | Symbol | Color |
|--------|--------|-------|
| Confirmed | ● | Green |
| Worked, not confirmed | ◐ | Purple |
| Not worked | ○ | Gray outline |

### State Grid Cell

```
┌───────┐
│  CA   │
│   ●   │     ← status indicator
│ 2024  │     ← last year worked
└───────┘
```

### Screen Components

### Award Card (Summary)

```
┌─────────────────────────┐
│                         │
│      [Icon]             │
│      DXCC               │
│                         │
│       89                │
│      /100               │
│                         │
│  ████████████░░░░░░░░░  │
│                         │
│      89%                │
│   11 needed             │
│                         │
└─────────────────────────┘
```

### Missing Entity Grid

```
┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐
│ AL  │ │ AK  │ │ AZ  │ │ AR  │ │ DE  │ │ HI  │ │ ID  │ │ IA  │
│  ◐  │ │  ◐  │ │  ○  │ │  ○  │ │  ○  │ │  ○  │ │  ○  │ │  ○  │
│     │ │     │ │     │ │     │ │     │ │     │ │     │ │     │
└─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘
       ◐ = worked   ○ = not worked
```

### Entity Map

- **USA Map**: Color-coded by state (worked/confirmed/missing)
- **World Map**: Color-coded by DXCC entity

---

## Integration with Statistics Dashboard

Awards progress can appear as summary cards on the main dashboard:

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  WAS 32/50  │  │ DXCC 89/100 │  │  POTA 12    │
│  ██████░░░  │  │ ████████░░  │  │  ██░░░░░░  │
└─────────────┘  └─────────────┘  └─────────────┘
```

---

## Implementation Steps

- [ ] Define award entity reference data (states, DXCC list, parks list)
- [ ] Create award progress query functions
- [ ] Build dark theme base styles
- [ ] Build award stat card component
- [ ] Build progress bar component
- [ ] Build entity grid component
- [ ] Build missing entities list component
- [ ] Integrate US states SVG map
- [ ] Integrate world map SVG for DXCC
- [ ] Add POTA/SOTA/IOTA parsing from QTH field
- [ ] Add confirmation status display
- [ ] Link awards to statistics dashboard
- [ ] Add filter controls (band, mode)
- [ ] Add exportable award summary (PDF)

## UI/UX Notes

- Use `egui` with dark theme configured globally
- Charts: consider `egui_plot` for native rendering, or SVG-based for maps
- Panels should have subtle borders and consistent padding (16px)
- Progress bars should animate on load
- Entity grids should be responsive (wrap on smaller screens)
- Maps can be simplified SVG (US states: 50 paths; DXCC: ~340 paths)

---

## Open Questions

1. Should we support custom/unofficial awards (e.g., "Worked All VK")?
2. Should POTA/SOTA references auto-detect from QTH or require manual entry?
3. Should the map be interactive (click state → show QSOs)?
4. Should we track partial progress toward 5BWAS per-band?
5. Should LoTW sync automatically update award progress?
