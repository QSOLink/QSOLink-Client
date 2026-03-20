# Design Document: QSO Globe/Map View

## Overview

Visualize all contacts on an interactive globe or world map, plotting station locations based on available data (Grid Square, city/state/country, or callsign prefix lookup).

---

## Feature Description

Display all logged QSOs as pins/markers on a 3D globe or 2D map:
- **Grid Square**: Precise Maidenhead locator → lat/lon conversion
- **City/State/Country**: Geocode QTH field to coordinates
- **Callsign Prefix**: Lookup approximate location from callsign database
- **Fallback**: Show on world map at DXCC entity centroid

---

## FOSS Options Research (BSD-2 Compatible)

### Option A: CesiumJS (WebView)

[CesiumJS](https://cesium.com/platform/cesiumjs/) — Open source WebGL globe.

| Pros | Cons |
|------|------|
| True 3D globe | Requires WebView |
| High quality | Larger dependency |
| KML/GeoJSON support | GPU intensive |
| Active community | |
| **License**: Apache 2.0 ✅ | |

### Option B: Leaflet + OpenStreetMap (WebView)

[Leaflet](https://leafletjs.com/) — Lightweight 2D map library.

| Pros | Cons |
|------|------|
| Simple, proven | 2D only (no globe) |
| Many plugins | Less impressive |
| Works offline with tiles | Tile server dependency |
| Small footprint | |
| **License**: BSD 2-Clause ✅ | |

**Note**: OpenStreetMap tiles use ODbL (requires attribution) ✅

### Option C: Marble Widget (Qt/Native)

[Marble](https://marble.kde.org/) — KDE's virtual globe widget.

| Pros | Cons |
|------|------|
| Native, no WebView | Qt/KDE dependency |
| OpenStreetMap data | Integration complexity |
| Works offline | Less polished visuals |
| **License**: GPL/LGPL ❌ | **INCOMPATIBLE** |

### Option D: Globe.gl (WebView)

[Globe.gl](https:// globe.gl/) — React/Three.js based 3D globe.

| Pros | Cons |
|------|------|
| Beautiful 3D | Requires WebView |
| Arcs between points | Heavy |
| Easy data binding | |
| **License**: ISC ✅ | |

### Option E: Custom Canvas Rendering

Pure Rust with `wgpu` or `plotters`.

| Pros | Cons |
|------|------|
| No WebView | Very complex |
| Fully native | Limited globe features |
| Works offline | Significant dev time |
| **License**: MIT/Apache 2.0 ✅ | |

---

## License Compatibility Summary

| Option | License | BSD-2 Compatible |
|--------|---------|-------------------|
| CesiumJS | Apache 2.0 | ✅ |
| Leaflet | BSD 2-Clause | ✅ |
| Marble | GPL/LGPL | ❌ |
| Globe.gl | ISC | ✅ |
| Custom (wgpu) | MIT/Apache 2.0 | ✅ |

---

## Recommendation

**Option B (Leaflet)** for simplest integration, or **Option A (CesiumJS)** for 3D globe. Both are fully BSD-2 / Apache-2 / MIT compatible.

For an egui app, embed a WebView with the chosen library. Cross-platform webview support via `eframe` + webview or `webview` crate.

---

## Data Sources for Location

### Grid Square → Lat/Lon

```rust
fn grid_square_to_latlon(grid: &str) -> Option<(f64, f64)> {
    // Maidenhead 4 or 6 character grid
    // Returns center of grid square
}
```

### City/State → Lat/Lon

| Method | Source |
|--------|--------|
| Geocoding API | Requires internet; limited offline |
| Embedded database | Larger binary; works offline |
| Manual entry | User specifies lat/lon |

**Recommended:** Embed a lightweight city database (e.g., GeoLite2 city) for offline use.

### Callsign Prefix → Approximate Location

| Database | Coverage | Format |
|----------|----------|--------|
| QRZ.com | Subscription | API |
| HamDB | Free | XML/JSON |
| Local prefix database | Embedded | CSV |

**Recommended:** Embed `cty.dat` (used by Club Log, DX Watch) — comprehensive prefix → country/DXCC mapping. Use centroid of DXCC entity for location.

---

## UI Design

```
┌─────────────────────────────────────────────────────────────────────────┐
│  QSO Globe / Map                                                  [X]  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  View: [Globe ●] [Map ○]    Filter: [All ▾]    [⟲ Sync]    [⚙]       │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                                                                  │   │
│  │                    🌍 3D Globe / 🗺️ 2D Map                      │   │
│  │                                                                  │   │
│  │         ○ Japan (89)     ● Germany (45)                        │   │
│  │              ○                ●                                  │   │
│  │      ○                      ●● Australia (32)                  │   │
│  │                  ●● USA (423)                                   │   │
│  │                      ●●●                                        │   │
│  │                        ●●                                       │   │
│  │                     ●●●●                                        │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─ Legend ──────────────────────────────────────────────────────────┐ │
│  │  ● Worked  ○ Needed (for award)  ▲ Station Location              │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  ┌─ Selected Contact ────────────────────────────────────────────────┐ │
│  │  N6ABC - John Smith (Los Angeles, CA)                            │ │
│  │  20m SSB │ 2026-03-15 14:30 UTC │ Grid: DM04                     │ │
│  │  [Center on Globe]  [View Details]                               │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Marker Clustering

When many QSOs are in the same region, cluster markers:

| Zoom Level | Display |
|------------|---------|
| World | Cluster by country |
| Continent | Cluster by region |
| Country | Cluster by city |
| City | Individual pins |

---

## Globe Interactions

| Interaction | Result |
|-------------|--------|
| Click marker | Show contact details popup |
| Hover marker | Show callsign tooltip |
| Drag | Rotate globe |
| Scroll | Zoom in/out |
| Double-click | Zoom to location |
| Right-click | Context menu (center, view contact, etc.) |

---

## Map Layers

| Layer | Source | Description |
|-------|--------|-------------|
| Base map | OpenStreetMap / Natural Earth | Geographic context |
| DXCC entities | Embedded | Color-coded countries |
| Grid squares | Calculated | Maidenhead grid overlay (optional) |
| QSO markers | Database | Contact locations |
| Station marker | Config | Home QTH |

---

## Data Model

### Location Resolution

```rust
pub struct QsoLocation {
    pub contact_id: Uuid,
    pub callsign: String,
    
    // Resolution method (in order of preference)
    pub location_type: LocationType, // GridSquare, Geocoded, Prefix, DXCC_Centroid
    
    // Coordinates
    pub latitude: f64,
    pub longitude: f64,
    
    // Confidence level
    pub accuracy: Accuracy, // Exact, Approximate, Country
}

pub enum LocationType {
    GridSquare,      // Maidenhead 4/6 char
    GeocodedQTH,    // City geocoded
    CallsignPrefix, // From prefix database
    DXCCEntity,     // DXCC centroid
}

pub enum Accuracy {
    Exact,      // Grid square
    High,       // Geocoded city
    Medium,     // Callsign prefix
    Low,        // DXCC centroid
}
```

---

## Location Resolution Pipeline

```
For each contact:
    1. If grid_square exists:
         → Convert to lat/lon (exact)
    2. Else if QTH contains city/country:
         → Geocode against embedded city DB
    3. Else if callsign prefix lookup succeeds:
         → Get DXCC entity centroid
    4. Else:
         → Mark as "unknown location"
```

---

## Implementation Steps

- [ ] Research WebView integration in egui (webview crate or eframe)
- [ ] Evaluate CesiumJS vs Leaflet for this use case
- [ ] Create location resolution module
- [ ] Implement Grid Square → Lat/Lon conversion
- [ ] Embed callsign prefix database (cty.dat or similar)
- [ ] Implement callsign → DXCC lookup
- [ ] Build embedded WebView with globe/map library
- [ ] Implement marker clustering
- [ ] Add contact detail popup on click
- [ ] Implement zoom/pan controls
- [ ] Add legend and filters
- [ ] Optimize for 10k+ contacts
- [ ] Test offline functionality

---

## Open Questions

1. Should we require internet for geocoding, or embed a full city database?
2. How to handle mobile/field use where internet is unavailable?
3. Should we show arcs connecting home station to contacts?
4. Should we color-code markers by band, mode, or date?
5. How to handle POTA/SOTA locations differently from regular QTH?
