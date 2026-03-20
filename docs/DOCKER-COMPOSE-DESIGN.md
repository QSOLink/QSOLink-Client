# Design Document: Docker Compose Setup

## Overview

Provide example Docker Compose configurations for common QSOLink deployments, including database and transceiver control components.

---

## Scope

This document provides:
1. Example `docker-compose.yml` files
2. Configuration guidance for each component
3. Instructions for exposing serial/USB devices to containers
4. Security considerations
5. Troubleshooting tips

---

## Components

| Component | Purpose | Image |
|-----------|---------|-------|
| **QSOLink Server** | REST API backend | Custom build |
| **PostgreSQL** | Primary database | `postgres:16` |
| **MySQL** | Alternative database | `mysql:8` |
| **rigctld** | Hamlib transceiver control | Custom or `ghcr.io/holyhz/hamlib` |

---

## Example: QSOLink + PostgreSQL

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    container_name: qsolink-db
    restart: unless-stopped
    environment:
      POSTGRES_DB: qsolink
      POSTGRES_USER: qsolink
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backups:/backups
    secrets:
      - db_password
    networks:
      - qsolink-net
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U qsolink"]
      interval: 10s
      timeout: 5s
      retries: 5

  qsolink-server:
    image: qsolink/server:latest
    container_name: qsolink-server
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgresql://qsolink:$${DB_PASSWORD}@postgres:5432/qsolink
    volumes:
      - ./config:/app/config
    depends_on:
      postgres:
        condition: service_healthy
    networks:
      - qsolink-net

networks:
  qsolink-net:
    driver: bridge

volumes:
  postgres_data:

secrets:
  db_password:
    file: ./secrets/db_password.txt
```

---

## Example: QSOLink + MySQL

```yaml
version: '3.8'

services:
  mysql:
    image: mysql:8
    container_name: qsolink-db
    restart: unless-stopped
    environment:
      MYSQL_ROOT_PASSWORD_FILE: /run/secrets/root_password
      MYSQL_DATABASE: qsolink
      MYSQL_USER: qsolink
      MYSQL_PASSWORD_FILE: /run/secrets/db_password
    volumes:
      - mysql_data:/var/lib/mysql
      - ./backups:/backups
    secrets:
      - root_password
      - db_password
    networks:
      - qsolink-net
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost", "-u", "root"]
      interval: 10s
      timeout: 5s
      retries: 5
    command: --default-authentication-plugin=mysql_native_password

  qsolink-server:
    image: qsolink/server:latest
    container_name: qsolink-server
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: mysql://qsolink:$${DB_PASSWORD}@mysql:3306/qsolink
    depends_on:
      mysql:
        condition: service_healthy
    networks:
      - qsolink-net

networks:
  qsolink-net:
    driver: bridge

volumes:
  mysql_data:

secrets:
  root_password:
    file: ./secrets/root_password.txt
  db_password:
    file: ./secrets/db_password.txt
```

---

## Example: QSOLink + rigctld

For transceiver control via CAT/CIV, run `rigctld` in a separate container:

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    # ... (see above)

  qsolink-server:
    # ... (see above)

  rigctld:
    image: ghcr.io/holyhz/hamlib:latest
    container_name: qsolink-rigctld
    restart: unless-stopped
    devices:
      - /dev/serial/by-id/usb-FTDI_FT232R_USB_UART_A50285BI-if00-port0:/dev/ttyUSB0
    environment:
      RIG_MODEL: "1024"  # Icom IC-7300
      RIG_SERIAL: "/dev/ttyUSB0"
      RIG_BAUD: "115200"
    ports:
      - "4532:4532"  # rigctld port
    networks:
      - qsolink-net
    command: rigctld -m 1024 -r /dev/ttyUSB0 -s 115200 -T 0.0.0.0 -t 4532
```

---

## Serial/USB Device Passthrough

### Identifying Your Device

```bash
# List USB devices
ls -la /dev/serial/by-id/

# List USB devices by path
ls -la /dev/serial/by-path/

# Example output:
# usb-FTDI_FT232R_USB_UART_A50285BI-if00-port0 -> ../../ttyUSB0
```

### Options for Device Access

#### Option 1: Device Path (Simple)

```yaml
devices:
  - /dev/ttyUSB0:/dev/ttyUSB0
```

**Pros:** Simple
**Cons:** Device path may change on reboot

#### Option 2: By-ID (Recommended)

```yaml
devices:
  - /dev/serial/by-id/usb-FTDI_FT232R_USB_UART_A50285BI-if00-port0:/dev/ttyUSB0
```

**Pros:** Persistent across reboots
**Cons:** None significant

#### Option 3: By-Path

```yaml
devices:
  - /dev/serial/by-path/pci-0000:00:14.0-usb-0:1:1.0-port0:/dev/ttyUSB0
```

**Pros:** Specific to physical port
**Cons:** Changes if USB topology changes

### Docker Compose Configuration

```yaml
services:
  rigctld:
    # ... other config
    devices:
      - /dev/serial/by-id/usb-FTDI_FT232R_USB_UART_A50285BI-if00-port0:/dev/ttyUSB0
    # Required for direct hardware access
    privileged: true
```

### Linux Permissions

```bash
# Add user to dialout group (required for serial access)
sudo usermod -aG dialout $USER

# Verify
groups $USER
```

### USB Permission udev Rule (Optional)

Create `/etc/udev/rules.d/99-ham-radio.rules`:

```rules
# FTDI devices
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", MODE="0666", GROUP="dialout"

# Icom CI-V
SUBSYSTEM=="tty", ATTRS{idVendor}=="10c4", MODE="0666", GROUP="dialout"

# Kenwood
SUBSYSTEM=="tty", ATTRS{idVendor}=="0c26", MODE="0666", GROUP="dialout"
```

Then reload udev:

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

---

## Security Considerations

### Database Passwords

**Always use Docker secrets for passwords:**

```bash
# Create secrets directory
mkdir -p secrets

# Generate secure passwords
openssl rand -base64 32 > secrets/db_password.txt
openssl rand -base64 32 > secrets/root_password.txt

# Restrict permissions
chmod 600 secrets/*.txt
```

### Network Security

```yaml
# Use internal network only
networks:
  qsolink-net:
    internal: true  # No external access
```

### TLS/SSL

For production, terminate TLS at a reverse proxy:

```yaml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "443:443"
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./certs:/etc/nginx/certs
    depends_on:
      - qsolink-server
```

---

## Environment Variables Reference

### PostgreSQL

| Variable | Description | Required |
|----------|-------------|----------|
| `POSTGRES_DB` | Database name | Yes |
| `POSTGRES_USER` | Database user | Yes |
| `POSTGRES_PASSWORD` | Database password | Yes* |
| `POSTGRES_PASSWORD_FILE` | Password file path | Yes* |

### MySQL

| Variable | Description | Required |
|----------|-------------|----------|
| `MYSQL_DATABASE` | Database name | Yes |
| `MYSQL_USER` | Database user | Yes |
| `MYSQL_PASSWORD` | Database password | Yes* |
| `MYSQL_PASSWORD_FILE` | Password file path | Yes* |

### QSOLink Server

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | Database connection string | Yes |
| `JWT_SECRET` | JWT signing secret | Yes |
| `RIGCTLD_HOST` | rigctld host | No |
| `RIGCTLD_PORT` | rigctld port | No |

---

## Rig Model Reference

Common Hamlib model numbers:

| Model | ID | Radio |
|-------|----|-------|
| Icom IC-7300 | 1024 | Icom IC-7300 |
| Icom IC-705 | 1029 | Icom IC-705 |
| Icom IC-9700 | 1031 | Icom IC-9700 |
| Yaesu FT-991A | 135 | Yaesu FT-991A |
| Kenwood TS-480 | 305 | Kenwood TS-480 |
| Elecraft K3 | 2041 | Elecraft K3 |

See `rigctl --list` for complete list.

---

## Common Issues

### Database Connection Refused

```bash
# Check if database is running
docker compose ps

# Check logs
docker compose logs postgres

# Verify connection string
docker compose exec qsolink-server env | grep DATABASE
```

### Serial Device Not Found

```bash
# Check device exists on host
ls -la /dev/serial/by-id/

# Check container can see device
docker compose exec rigctld ls -la /dev/ttyUSB0

# Check kernel messages
dmesg | grep -i usb
dmesg | grep -i tty
```

### Permission Denied on Serial Port

```bash
# Verify user is in dialout group
groups $USER

# If not, log out and back in, or run:
newgrp dialout

# Or use sudo for docker
sudo docker compose up -d
```

### rigctld Not Connecting to Radio

```bash
# Test from host
rigctl -m 1024 -r /dev/ttyUSB0 -s 115200

# Test from container
docker compose exec rigctld rigctl -m 1024 -r /dev/ttyUSB0 -s 115200

# Check firewall
sudo ufw status
```

---

## File Structure

```
qsolink/
├── docker-compose.yml
├── docker-compose.postgres.yml
├── docker-compose.mysql.yml
├── docker-compose.rigctld.yml
├── .env.example
├── nginx/
│   └── nginx.conf
├── certs/
│   ├── fullchain.pem
│   └── privkey.pem
├── backups/
│   └── .gitkeep
├── secrets/
│   ├── db_password.txt
│   └── root_password.txt
└── config/
    └── qsolink.toml
```

---

## Next Steps

After setup:

1. Configure QSOLink client to connect to server
2. Set up operator profiles
3. Configure rigctld settings in QSOLink
4. Test transceiver control
5. Set up automated backups

---

## Open Questions

1. Should we provide pre-built Docker images for QSOLink server?
2. Should we support Docker Compose profiles for optional components?
3. Should we include Traefik/Caddy for automatic HTTPS?
4. Should we provide Helm charts for Kubernetes deployment?
