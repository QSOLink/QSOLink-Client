# Design Document: Multi-User Remote Database Support

## Overview

Support multiple users connecting to a shared remote database (PostgreSQL/MySQL) with authentication, access control, and data isolation.

---

## Use Cases

| Scenario | Description |
|----------|-------------|
| **Club Station** | Multiple club members share one database; all see all contacts |
| **Family** | Husband and wife share station; filter by operator profile |
| **Contest Team** | Multi-operator contest with shared log; separate operator tracking |
| **Remote Access** | Operator connects to home database from field laptop |

---

## Data Model

### Users Table

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'operator',  -- 'admin', 'operator', 'readonly'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP
);
```

### Roles & Permissions

| Role | Read Contacts | Write Contacts | Manage Users | Manage Database |
|------|--------------|----------------|--------------|-----------------|
| `admin` | ✅ | ✅ | ✅ | ✅ |
| `operator` | ✅ | ✅ | ❌ | ❌ |
| `readonly` | ✅ | ❌ | ❌ | ❌ |

### Operator-User Link

```sql
CREATE TABLE user_operators (
    user_id TEXT REFERENCES users(id),
    operator_id TEXT REFERENCES operators(id),
    PRIMARY KEY (user_id, operator_id)
);
```

---

## Connection Flow

```
┌─────────────────────────────────────────────────────────────┐
│                        QSOLink Client                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─ Login ──────────────────────────────────────────────┐  │
│  │                                                           │  │
│  │  Database: [mysql://server/qsolog            ]         │  │
│  │  Username: [admin                    ]                 │  │
│  │  Password: [••••••••                ]                 │  │
│  │                                                           │  │
│  │  [ ] Remember credentials                                      │  │
│  │                                                            │  │
│  │                        [Connect]  [Cancel]                │  │
│  │                                                            │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Connection String Formats

```toml
# PostgreSQL
postgresql://user:password@hostname:5432/database

# MySQL
mysql://user:password@hostname:3306/database
```

---

## Server Requirements

The remote database server must:

1. **Host a QSOLink server** — lightweight REST API or direct database access
2. **Handle authentication** — validate user credentials
3. **Enforce access control** — role-based permissions
4. **Support offline clients** — sync when reconnected

### Server Architecture Options

| Option | Description | Pros | Cons |
|--------|-------------|------|------|
| **Direct DB Access** | Clients connect directly to MySQL/PostgreSQL | Simple | Security risks, no offline support |
| **REST API** | Lightweight web server with JSON API | Secure, works with sync | More complex |
| **GraphQL** | Flexible query API | Powerful queries | Overkill for this use case |

### Recommended: REST API

A simple REST API server that:
- Authenticates users (JWT tokens)
- Serves contact CRUD operations
- Handles sync operations (bidirectional)
- Can be deployed alongside existing PostgreSQL/MySQL

---

## API Design

### Endpoints

```
Authentication:
POST   /api/auth/login     → { token, user, operators[] }
POST   /api/auth/logout    → {}

Contacts:
GET    /api/contacts       → [Contact] (paginated, filtered by operator)
GET    /api/contacts/:id   → Contact
POST   /api/contacts       → Contact
PUT    /api/contacts/:id   → Contact
DELETE /api/contacts/:id   → {}

Sync:
GET    /api/sync/pull      → { contacts[], last_sync }
POST   /api/sync/push      → { conflicts[] }

Users (admin only):
GET    /api/users
POST   /api/users
PUT    /api/users/:id
DELETE /api/users/:id
```

### Authentication

```rust
// Login request
struct LoginRequest {
    username: String,
    password: String,
}

// Login response
struct LoginResponse {
    token: String,          // JWT
    expires_at: i64,        // Unix timestamp
    user: User,
    operators: Vec<Operator>,
    last_sync: Option<i64>, // Last sync timestamp
}
```

---

## Offline Behavior

| Scenario | Behavior |
|----------|----------|
| No connection | Use local SQLite cache |
| Sync available | Bidirectional merge |
| Conflicts | Flag for user resolution |
| Token expired | Re-authenticate |

---

## Security Considerations

| Concern | Mitigation |
|---------|------------|
| Password storage | bcrypt or argon2 hashing |
| Transport | TLS/SSL required |
| Token expiry | Short-lived JWT (1 hour) + refresh token |
| SQL injection | Parameterized queries only |
| Rate limiting | Per-user request limits |

---

## Database Schema (Server)

```sql
-- Server-side schema (PostgreSQL/MySQL)

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'operator',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP
);

CREATE TABLE operators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    callsign VARCHAR(15) UNIQUE NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    license_class VARCHAR(20),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE stations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operator_id UUID REFERENCES operators(id),
    name VARCHAR(100),
    callsign VARCHAR(15),
    grid_square VARCHAR(6),
    qth VARCHAR(255),
    latitude DECIMAL(9,6),
    longitude DECIMAL(9,6),
    is_default BOOLEAN DEFAULT FALSE
);

CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operator_id UUID REFERENCES operators(id),
    station_id UUID REFERENCES stations(id),
    
    -- Standard QSO fields
    callsign VARCHAR(15) NOT NULL,
    date DATE NOT NULL,
    time_on TIME NOT NULL,
    time_off TIME,
    band VARCHAR(10),
    mode VARCHAR(10),
    frequency DECIMAL(10,3),
    rst_sent VARCHAR(4),
    rst_recv VARCHAR(4),
    name VARCHAR(100),
    qth VARCHAR(255),
    grid_square VARCHAR(6),
    notes TEXT,
    
    -- LoTW
    lotw_sent BOOLEAN DEFAULT FALSE,
    lotw_confirmed BOOLEAN DEFAULT FALSE,
    
    -- Sync metadata
    device_id VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_deleted BOOLEAN DEFAULT FALSE,
    
    -- Indexes
    INDEX idx_operator (operator_id),
    INDEX idx_date (date),
    INDEX idx_callsign (callsign)
);
```

---

## Implementation Components

### Client-Side

| Component | Description |
|-----------|-------------|
| Auth module | Login, logout, token refresh |
| Remote DB client | HTTP client for API calls |
| Sync engine | Bidirectional merge logic |
| Offline cache | Local SQLite with sync state |
| Conflict resolver | UI for manual conflict resolution |

### Server-Side

| Component | Description |
|-----------|-------------|
| Auth handler | User registration, login, JWT issuance |
| Contact CRUD | Standard REST operations |
| Sync handler | Pull/push with conflict detection |
| Admin panel | User management API |

---

## Sync Protocol

Based on the [Sync Design Document](SYNC-DESIGN.md):

```
1. Client sends: { last_sync: timestamp, device_id }
2. Server responds: { 
       contacts: [...],  // New/modified since last_sync
       deleted: [uuid, ...],  // Deleted IDs
       server_time: timestamp
   }
3. Client sends: { contacts: [...], conflicts: [...] }
4. Server processes, returns: { success: true, conflicts_resolved: [...] }
```

---

## Client Settings Storage

```toml
# Stored locally on client

[connection]
type = "remote"  # or "local"
server_url = "https://qsolink.example.com"

[connection.credentials]
username = "n6abc"
# Password stored in system keychain, not here

[connection.last_operator]
id = "uuid-here"

[connection.sync]
last_sync = 1709308800
```

---

## Implementation Steps

### Client

- [ ] Add users/roles table schema
- [ ] Implement login/logout UI
- [ ] Add JWT token storage (secure)
- [ ] Create remote DB client module
- [ ] Add "remote" connection type to settings
- [ ] Implement sync engine with remote API
- [ ] Add conflict resolution UI
- [ ] Handle offline mode gracefully
- [ ] Token refresh logic

### Server

- [ ] Set up REST API project structure
- [ ] Implement user registration/login
- [ ] Add JWT middleware
- [ ] Implement contact CRUD endpoints
- [ ] Implement sync endpoints
- [ ] Add admin endpoints for user management
- [ ] Set up database migrations
- [ ] Add rate limiting
- [ ] TLS configuration
- [ ] Documentation / deployment guide

---

## Open Questions

1. Should the server be bundled with QSOLink or separate?
2. Do we need role-based UI changes (e.g., hide admin features)?
3. Should we support OAuth (Google, GitHub) for login?
4. How to handle password reset?
5. Should the server support WebSocket for real-time sync?
6. What is the expected concurrent user limit for MVP?
