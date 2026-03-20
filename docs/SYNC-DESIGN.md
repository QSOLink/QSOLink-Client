# User Story: Offline-First Sync with Multi-Node Merge

## Title

As a DXpeditioner, I want to sync my log between home and multiple field devices so I can maintain a unified contact database across disconnected environments.

---

## Story

**As a** ham radio operator conducting field operations from remote locations,

**I want to** work with a complete, self-contained copy of my logbook on any device,

**So that** I can log contacts independently without requiring network connectivity to my home server, regardless of how many devices I have in the field.

---

## Background & Motivation

Typical operators may have:
- A **base station** (desktop) always connected to home server
- A **laptop** for portable / field operations
- A **field computer** for major DXpeditions
- Additional devices for multi-operator contests or club stations

All devices should be able to:
- Operate fully offline
- Sync bidirectionally when connectivity is available
- Merge contacts without data loss or duplication
- Scale from 2 devices to 8+ without architectural changes

---

## Assumptions

1. **Server** is always online and reachable when devices have connectivity
2. **Devices** may be offline for days, weeks, or months
3. **Connectivity** may be intermittent, asymmetric (device A can reach server, device B cannot), or delayed
4. **Clock accuracy** varies — devices may have incorrect timestamps; cannot rely solely on time for ordering
5. **Identity** — each device has a unique identifier; contacts record which device created them

---

## Definitions

| Term | Definition |
|------|------------|
| **Node** | Any device (server, laptop, field computer) that stores contacts |
| **Contact** | A logged QSO with callsign, date, time, band, mode, and other fields |
| **Sync** | Bidirectional transfer of new contacts between nodes |
| **Merge** | Process of integrating contacts from one node into another |
| **Conflict** | Same contact exists on multiple nodes with different data |
| **Device ID** | Unique identifier for each logging device |

---

## Core Use Cases

### UC-1: Field Operation

**Scenario:** Operator takes a laptop to a remote island with no internet.

| Step | Action |
|------|--------|
| 1 | Before departure, sync laptop with server to get latest contacts |
| 2 | Travel to island, operate for days/weeks offline |
| 3 | Log all contacts to local SQLite database |
| 4 | (Optional) Occasional sync if satellite connectivity is available |

**Acceptance Criteria:**
- [ ] Laptop contains complete copy of all contacts from server at sync time
- [ ] New contacts are persisted locally with device ID and creation timestamp
- [ ] No network dependency for logging

---

### UC-2: Return and Merge

**Scenario:** Operator returns home with laptop after weeks of field operation.

| Step | Action |
|------|--------|
| 1 | Connect laptop to home network |
| 2 | QSOLink detects server is available |
| 3 | Bidirectional sync occurs |
| 4 | Conflicts are resolved automatically or flagged for manual review |

**Acceptance Criteria:**
- [ ] All contacts from laptop are merged into server
- [ ] All contacts added to server while laptop was offline are added to laptop
- [ ] No duplicates created (callsign + date + time + band is unique)
- [ ] User is notified of any conflicts that required manual resolution

---

### UC-3: Multiple Simultaneous Expeditions

**Scenario:** Operator has laptop in Pacific (Island A) and field computer in Caribbean (Island B), both offline for weeks.

| Step | Action |
|------|--------|
| 1 | Both devices sync with server before departure |
| 2 | Laptop operator adds 50 contacts on Island A |
| 3 | Field computer operator adds 75 contacts on Island B |
| 4 | Both return home at different times |

**Acceptance Criteria:**
- [ ] First device to sync uploads its contacts without conflict
- [ ] Second device to sync detects no conflicts (different contacts)
- [ ] Both sets of contacts appear in unified database
- [ ] Server maintains provenance (which device created each contact)

---

### UC-4: Multi-Week Expedition with Intermittent Connectivity

**Scenario:** Operator returns home on weekends but laptop stays at remote island.

| Step | Action |
|------|--------|
| 1 | Laptop synced before leaving home |
| 2 | Week 1: 100 contacts logged on island |
| 3 | Weekend: Operator brings laptop home, syncs |
| 4 | Week 2: 80 contacts logged on island (including some from home station operators) |
| 5 | Weekend: Operator syncs again |

**Acceptance Criteria:**
- [ ] Each sync is incremental (only new contacts transferred)
- [ ] Contacts logged at home station during week appear on laptop after sync
- [ ] Operator can search/lookup home contacts while on island
- [ ] Sync is idempotent (running sync twice has no ill effect)

---

### UC-5: End-of-Expedition Final Merge

**Scenario:** DXpedition complete, laptop returning home permanently.

| Step | Action |
|------|--------|
| 1 | Operator performs final sync |
| 2 | All expedition contacts are merged into server |
| 3 | Laptop can be wiped or repurposed |

**Acceptance Criteria:**
- [ ] Server has 100% of all contacts ever created
- [ ] Laptop can optionally be cleared of contact data
- [ ] Audit trail shows which contacts came from which device

---

## Conflict Resolution Strategy

When the same contact exists on multiple nodes with different data:

### Automatic Resolution (Default)

| Rule | When Applied | Resolution |
|------|--------------|------------|
| **Duplicate Detection** | Contact with same CALLSIGN + DATE + TIME_ON exists | No merge; treated as same contact |
| **Last-Write-Wins** | Same field differs between nodes | Highest timestamp wins |
| **Field-Override** | For expedition contacts | Island/field device data takes precedence |

### Manual Resolution (Flagged)

| Scenario | Action |
|----------|--------|
| Same callsign, different times | Prompt user to confirm if same contact |
| Same callsign, different bands | Treat as separate contacts |
| Incomplete data on one node | Prefer complete record |

---

## Scalability Considerations

### 2-3 Devices

**Architecture:** Central server + last-write-wins

Simple and sufficient. Conflicts are rare enough that automatic resolution handles nearly all cases.

### 8+ Devices / Simultaneous Expeditions

**Architecture:** CRDT-based or Vector Clocks

| Challenge | Solution |
|-----------|----------|
| Concurrent edits | CRDT ensures automatic merge without conflict |
| Partition tolerance | Each node can make progress independently |
| Causality tracking | Vector clocks record "happened-before" relationships |
| Scale | System handles any number of nodes without architectural change |

### Migration Path

1. **Phase 1:** Implement simple server + device sync with last-write-wins
2. **Phase 2:** Add device IDs and conflict logging
3. **Phase 3:** Upgrade to CRDT-based merge when needed

---

## Technical Design Options

### Option A: Simple Server Sync (Recommended for 2-5 devices)

- REST API for sync operations
- Server is source of truth for timestamps
- Last-write-wins for conflicts
- Device ID stored on each contact

**Pros:** Simple to implement, easy to understand
**Cons:** Requires server for conflict resolution; may lose data in complex merge scenarios

### Option B: CRDT-Based Sync (Recommended for 5+ devices)

- Each contact is an immutable event with unique ID
- Contacts can be merged automatically across any number of nodes
- Server aggregates but doesn't dictate ordering

**Pros:** Handles any topology; no data loss; works fully offline
**Cons:** More complex to implement; larger payload per contact

### Option C: Git-Like Model (Maximum Control)

- Explicit branches per device
- Merge requests when syncing
- Manual conflict resolution by user

**Pros:** User has full control; audit trail
**Cons:** High user burden; not practical for casual operation

---

## Recommended Approach

**Start with Option A (Simple Server Sync)**, designed to evolve to Option B (CRDT) when needed.

Key design principles:
1. Each contact has a UUID regardless of device
2. Device ID is recorded on creation
3. Timestamps are in UTC
4. Server maintains canonical timestamp
5. Conflict resolution is configurable (field-wins, server-wins, manual)

---

## Future Development Tasks

- [ ] Phase 1: Sync API design and server endpoints
- [ ] Phase 1: Device registration and authentication
- [ ] Phase 1: Bidirectional sync with conflict detection
- [ ] Phase 2: Duplicate contact detection (CALLSIGN + DATE + TIME)
- [ ] Phase 2: Conflict resolution UI (show differences, let user pick)
- [ ] Phase 3: CRDT-based merge engine
- [ ] Phase 3: Vector clock tracking for causality
- [ ] Phase 3: Offline-first with eventual consistency

---

## Open Questions

1. Should the server be required for operation, or should devices work in "serverless" mode when disconnected?
2. How should contacts edited (not just added) be synced? Is editing allowed after creation?
3. Should contacts be deletable? If so, how is deletion synced?
4. What happens if a contact is logged on two devices with slightly different times (operator error)?
5. Should there be a maximum number of devices allowed, or unlimited?
