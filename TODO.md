# TODO / Coming Soon

### Logbook of the World (LoTW)
- [x] ARRL LoTW integration (check confirmations via lotw.arrl.org)
- [x] Mark contacts as lotw_submitted / lotw_confirmed
- [x] Periodic sync on startup and hourly while running
- [x] Station profile setup at first launch
- [x] Mode dropdown with ADIF-enumerated values
- [x] Maidenhead grid square validation
- [x] ADIF export with STATION_CALLSIGN, MY_GRIDSQUARE, SUBMODE for LoTW
- [ ] **TQSL direct integration** (sign and upload ADIF via TrustedQSL) — deferred to future release

### Additional Features
- [Multi-node sync with merge](docs/SYNC-DESIGN.md) — sync log between home server and field devices
- [Contact editing](docs/EDIT-CONTACT-DESIGN.md) — update existing contacts
- [Duplicate contact detection](docs/DUPLICATE-DETECTION-DESIGN.md) — prevent/log duplicate QSOs
- [Statistics dashboard](docs/STATISTICS-DASHBOARD-DESIGN.md) — QSOs per band/mode with charts
- [ARRL Awards tracking](docs/AWARDS-TRACKING-DESIGN.md) — progress toward WAS, DXCC, POTA, etc.
- [QSO Globe/Map view](docs/QSO-MAP-DESIGN.md) — plot contacts on a globe using Grid Square or location data
- [Operator profiles](docs/MULTI-USER-PROFILES-DESIGN.md) — multi-operator support with profile switching
- [Multi-user remote database](docs/MULTI-USER-REMOTE-DESIGN.md) — shared database with auth, roles, and remote sync
- [Docker compose documentation](docs/DOCKER-COMPOSE-DESIGN.md) — example setup with PostgreSQL/MySQL + rigctld
- [Backup/restore](docs/BACKUP-RESTORE-DESIGN.md) — automated backups, ADIF/JSON export, restore with merge
- [Custom fields](docs/CUSTOM-FIELDS-DESIGN.md) — user-defined QSO fields, preset templates, ADIF export
- [FCC ULS Database Integration](docs/FCC-ULS-DESIGN.md) — offline US callsign lookup via weekly full + daily delta sync
