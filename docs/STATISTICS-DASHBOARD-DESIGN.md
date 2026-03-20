# Design Document: Statistics Dashboard

## Overview

Provide visual analytics of QSO activity, including counts by band, mode, country, and progress toward awards. Dashboard updates in real-time as contacts are logged.

---

## Dashboard Sections

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Statistics Dashboard                                              [X] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Total QSOs: 1,247    Countries: 89/339    States: 32/50             │
│                                                                         │
│  ┌─────────────────────────┐  ┌─────────────────────────┐             │
│  │     QSOs by Band        │  │     QSOs by Mode        │             │
│  │                         │  │                         │             │
│  │  40m ████████  312     │  │  SSB ██████████  623    │             │
│  │  20m ███████   289     │  │  FT8 ███████    412     │             │
│  │  15m █████     201     │  │  CW  ████       156     │             │
│  │  10m ████      156     │  │  FM  ██          56    │             │
│  │  17m ███       112     │  │                         │             │
│  │  ...                    │  │                         │             │
│  └─────────────────────────┘  └─────────────────────────┘             │
│                                                                         │
│  ┌─────────────────────────┐  ┌─────────────────────────┐             │
│  │   QSOs by Country       │  │   QSOs Over Time        │             │
│  │                         │  │                         │             │
│  │  JA     ███  89        │  │      ▂▃▅▇█▇▅▃▂▃▅▇█▇     │             │
│  │  VK     ██   56        │  │  2024                   │             │
│  │  G      ██   48        │  │                         │             │
│  │  UA     █    41        │  │                         │             │
│  │  ...                    │  │                         │             │
│  └─────────────────────────┘  └─────────────────────────┘             │
│                                                                         │
│  ┌─────────────────────────┐                                          │
│  │   Activity Heatmap      │  [Awards Progress →]                     │
│  │   (by UTC hour)         │                                          │
│  │  00  06  12  18  00    │                                          │
│  │  ▓▓▓▓░░▓▓▓▓▓▓▓▓▓░░░░  │                                          │
│  └─────────────────────────┘                                          │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Chart Types

### Bar Charts

| Chart | Data Source | Sort |
|-------|-------------|------|
| QSOs by Band | COUNT GROUP BY band | Descending |
| QSOs by Mode | COUNT GROUP BY mode | Descending |
| QSOs by Country | COUNT GROUP BY country | Descending |
| QSOs by State | COUNT GROUP BY state | Descending |

### Line/Area Charts

| Chart | X-Axis | Y-Axis |
|-------|--------|--------|
| QSOs Over Time | Date (daily/weekly/monthly) | Count |
| DXCC Progress | Date | Cumulative countries |
| WAS Progress | Date | Cumulative states |

### Heatmaps

| Chart | X-Axis | Y-Axis | Color |
|-------|--------|--------|-------|
| Activity by UTC Hour | Hour (0-23) | Day of Week | Intensity = QSO count |

---

## Visual Design Reference: Grafana-Inspired

Reference: [Grafana Dashboards](https://grafana.com/grafana/dashboards/) — dark theme with high contrast, clear data hierarchy, subtle gridlines.

### Color Palette

| Element | Color | Hex |
|---------|-------|-----|
| Background | Dark slate | `#1a1c23` |
| Card/Panel background | Slightly lighter | `#22252b` |
| Primary accent | Teal/cyan | `#00b8d9` |
| Secondary accent | Purple | `#9f7aea` |
| Success/confirmed | Green | `#36b37e` |
| Warning | Amber | `#ffab00` |
| Error/missing | Red-orange | `#ff5630` |
| Text primary | Light gray | `#e0e0e0` |
| Text secondary | Muted gray | `#9e9e9e` |
| Border/divider | Subtle | `#373a40` |

### Panel Design

```
┌─────────────────────────────────────────────────────────────┐
│ ┌─ Panel Header ────────────────────────────────────────┐ │
│ │ [Icon] QSOs by Band                      [⋮] [⧉] [X] │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                             │
│   ████████████████  40m - 312 QSOs                        │
│   ██████████████    20m - 289 QSOs                        │
│   ██████████        15m - 201 QSOs                        │
│   ████████          10m - 156 QSOs                        │
│   ██████            17m - 112 QSOs                        │
│                                                             │
│   ▁▂▃▄▅▆▇█▇▆▅▄▃▂▁                                     │
│   Panel footer: Last updated 2 min ago                    │
└─────────────────────────────────────────────────────────────┘
```

### Typography

| Element | Font | Size | Weight |
|---------|------|------|--------|
| Panel title | System sans-serif | 14px | Medium (500) |
| Stat value (big numbers) | System sans-serif | 32px | Bold (700) |
| Stat label | System sans-serif | 12px | Regular (400) |
| Axis labels | Monospace | 11px | Regular (400) |
| Legend | System sans-serif | 12px | Regular (400) |

### Stat Panels (Summary Cards)

```
┌─────────────────────────────────────────────────┐
│                                                 │
│   QSOs Today          Total QSOs    Countries  │
│   ██ 23               ████████ 1,247  ██ 89   │
│   +12% vs yesterday                        /339 │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Layout Grid

- **Row height**: 200px
- **Panel gap**: 16px
- **Padding**: 16px inside panels
- **Border radius**: 4px
- **Box shadow**: subtle `0 2px 4px rgba(0,0,0,0.3)`

### Interactive Elements

| Element | Behavior |
|---------|----------|
| Hover on bar | Tooltip with exact value |
| Click on bar | Filter dashboard to that category |
| Drag to resize | Panels are resizable |
| Drag to reorder | Panels can be rearranged |
| Time range selector | Top bar: Last 15m, 30m, 1h, 6h, 24h, 7d, 30d, Custom |

---

## Filters

| Filter | Options |
|--------|---------|
| Date Range | Last 7 days, 30 days, 90 days, This year, All time, Custom |
| Band | All, or specific band |
| Mode | All, or specific mode |
| Country | All, or specific DXCC entity |

---

## Summary Cards

| Metric | Calculation |
|--------|-------------|
| Total QSOs | COUNT(*) |
| Unique Countries | COUNT(DISTINCT country) |
| Unique States | COUNT(DISTINCT state) |
| Total Grid Squares | COUNT(DISTINCT grid_square) |
| QSO Rate (/day) | COUNT(*) / days_active |
| Avg QSOs/Week | COUNT(*) / weeks |

---

## Data Model

### Query Functions

```rust
pub struct QsoStats {
    pub total_qsos: i64,
    pub unique_countries: i64,
    pub unique_states: i64,
    pub unique_grids: i64,
}

pub struct BandCount {
    pub band: String,
    pub count: i64,
}

pub struct ModeCount {
    pub mode: String,
    pub count: i64,
}

pub struct CountryCount {
    pub country: String,
    pub dxcc: i32,
    pub count: i64,
}

pub struct DailyCount {
    pub date: NaiveDate,
    pub count: i64,
}
```

---

## Implementation Options

### Option A: SQL Aggregation (Recommended for MVP)

Run aggregation queries at dashboard load.

```sql
SELECT band, COUNT(*) as count FROM contacts GROUP BY band ORDER BY count DESC;
```

**Pros:** Simple, accurate
**Cons:** Slow with large datasets (100k+ contacts)

### Option B: Pre-computed Statistics

Store aggregates in a `statistics` table, update on QSO add/edit/delete.

```sql
CREATE TABLE daily_stats (
    date DATE PRIMARY KEY,
    total_qsos INTEGER,
    UNIQUE(date)
);
```

**Pros:** Fast dashboard load
**Cons:** More complex update logic

### Option C: Cached Aggregates

Use SQLite `MATERIALIZED VIEW` or in-memory caching.

**Pros:** Balance of speed and simplicity
**Cons:** Cache invalidation complexity

---

## Charting Library Options

| Library | Pros | Cons |
|---------|------|------|
| **egui_plot** | Native Rust, no JS | Limited chart types |
| **plotters** | Pure Rust, good charts | Less polished UI |
| **plotly** | Feature-rich, proven | Requires webview |
| **chartjs via webview** | Best visuals | Platform complexity |

**Recommendation:** Start with **egui_plot** for native experience. For heatmaps and more complex charts, consider SVG-based custom components.

### Color Scheme (Theme)

All charts should use the Grafana-inspired palette:

```rust
struct Theme {
    background: Color32::from_rgb(0x1a, 0x1c, 0x23),
    panel_bg: Color32::from_rgb(0x22, 0x25, 0x2b),
    primary: Color32::from_rgb(0x00, 0xb8, 0xd9),
    secondary: Color32::from_rgb(0x9f, 0x7a, 0xea),
    success: Color32::from_rgb(0x36, 0xb3, 0x7e),
    warning: Color32::from_rgb(0xff, 0xab, 0x00),
    error: Color32::from_rgb(0xff, 0x56, 0x30),
    text_primary: Color32::from_rgb(0xe0, 0xe0, 0xe0),
    text_secondary: Color32::from_rgb(0x9e, 0x9e, 0x9e),
    border: Color32::from_rgb(0x37, 0x3a, 0x40),
}
```

---

## Awards Integration

The dashboard should link to an Awards screen (see next design doc) showing progress toward:

- **DXCC** — Work 100 countries
- **WAS** — Work All States
- **WAC** — Work All Continents
- **IOTA** — Islands On The Air
- **POTA** — Parks On The Air
- **SOTA** — Summits On The Air
- **GridDXCC** — Work 100 grid squares

Progress indicators can be embedded in dashboard summary cards.

---

## Implementation Steps

- [ ] Define dark theme constants
- [ ] Create statistics query functions in `db.rs`
- [ ] Build base panel component (card with title, actions, content)
- [ ] Build stat card component (big number + label + trend)
- [ ] Add bar chart component
- [ ] Add line/area chart component
- [ ] Add heatmap component (UTC activity)
- [ ] Implement date range filtering
- [ ] Implement band/mode/country filtering
- [ ] Add "Export Stats" (CSV)
- [ ] Performance test with 100k+ contacts
- [ ] Add awards progress mini-cards

## UI/UX Notes

- Global dark theme with consistent color palette
- Panel-based layout with drag-to-reorder capability (future)
- Loading skeletons while data fetches
- Responsive grid (2-3 columns on desktop, 1 on mobile)
- Tooltips on hover for all chart elements

---

## Open Questions

1. Should dashboard load cached stats or compute fresh each time?
2. Should there be a "favorite charts" feature to customize view?
3. Should QSO rate calculations exclude off-air periods?
4. Should the heatmap show local time or UTC?
