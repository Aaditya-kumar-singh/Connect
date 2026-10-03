# Deployment — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-DEVOPS-002`                           |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-DEVOPS-001, DOC-DEVOPS-003            |

---

## 1. Deployment Architecture

```mermaid
graph TB
    subgraph "Developer Machine"
        DEV[Local Dev]
    end

    subgraph "CI/CD (GitHub Actions)"
        CI[Build + Test]
        CD[Deploy]
    end

    subgraph "Production"
        subgraph "VPS / Container Host"
            BACKEND[Rust Backend Binary]
            PG[(PostgreSQL)]
            REDIS[(Redis)]
            TURN[coturn]
        end
        subgraph "Cloudflare"
            PAGES[Pages (Frontend)]
            CF_R2[R2 (Media)]
            CF_DNS[DNS + CDN + WAF]
        end
    end

    DEV -->|git push| CI
    CI -->|Build & test| CD
    CD -->|Deploy binary| BACKEND
    CD -->|Deploy static| PAGES
    CI -->|Run migrations| PG
```

## 2. Backend Deployment

### Option A: Direct Binary (Recommended for Single VPS)

```bash
# Build release binary
cargo build --release --target x86_64-unknown-linux-gnu

# Upload to server
scp target/release/ybm-connect user@server:/opt/ybm-connect/bin/

# Restart service
ssh user@server "sudo systemctl restart ybm-connect"
```

### Systemd Service File

```ini
# /etc/systemd/system/ybm-connect.service
[Unit]
Description=YBM Connect Backend
After=network.target postgresql.service redis.service
Requires=postgresql.service redis.service

[Service]
Type=simple
User=ybm
Group=ybm
WorkingDirectory=/opt/ybm-connect
ExecStart=/opt/ybm-connect/bin/ybm-connect
EnvironmentFile=/opt/ybm-connect/.env
Restart=always
RestartSec=5
StartLimitBurst=5
StartLimitIntervalSec=60

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/ybm-connect/data
PrivateTmp=true

# Resource limits
LimitNOFILE=65536
MemoryMax=2G

[Install]
WantedBy=multi-user.target
```

### Option B: Docker Container

```dockerfile
# Dockerfile
FROM rust:1.75-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ybm-connect /usr/local/bin/
EXPOSE 8080
CMD ["ybm-connect"]
```

```bash
# Build and run
docker build -t ybm-connect:latest .
docker run -d \
  --name ybm-connect \
  --restart unless-stopped \
  --env-file .env.production \
  -p 8080:8080 \
  ybm-connect:latest
```

## 3. Frontend Deployment

### Web Frontend (Cloudflare Pages)

```bash
# Build static assets
cd frontend/web
npm run build

# Deploy to Cloudflare Pages
npx wrangler pages deploy out --project-name ybm-connect
```

Or configure Cloudflare Pages to auto-deploy from the Git repository:
1. Connect GitHub repo to Cloudflare Pages
2. Set build command: `cd frontend/web && npm install && npm run build`
3. Set output directory: `frontend/web/out` (or `.next` for SSR)
4. Set environment variables (`NEXT_PUBLIC_API_URL`, `NEXT_PUBLIC_WS_URL`)

### Mobile App

```bash
# Build production APK/AAB
cd frontend/mobile
npx eas-cli build --platform android --profile production

# Submit to Google Play (after initial manual setup)
npx eas-cli submit --platform android
```

## 4. Database Migration (Production)

```bash
# Before deploying new backend version:

# 1. Backup database
pg_dump -h localhost -U ybm -d ybm_connect > backup_$(date +%Y%m%d_%H%M%S).sql

# 2. Run migrations
DATABASE_URL="postgres://..." sqlx migrate run --source backend/migrations

# 3. Verify migration
psql -h localhost -U ybm -d ybm_connect -c "\dt"

# 4. Deploy new backend version
# (migration must succeed before backend deployment)
```

**Rule:** Never deploy a backend version that requires a migration BEFORE the migration has run. Run migrations first, then deploy.

## 5. Deployment Checklist

### Pre-Deployment
- [ ] All tests pass in CI
- [ ] Database migration tested (up AND down)
- [ ] Database backup taken
- [ ] Environment variables updated if new ones added
- [ ] Rollback plan documented
- [ ] No breaking API changes (or version bumped)

### During Deployment
- [ ] Run database migration
- [ ] Verify migration success
- [ ] Deploy backend binary/container
- [ ] Verify `/health` returns 200
- [ ] Verify `/ready` returns 200 (all dependencies OK)
- [ ] Deploy frontend (if changed)

### Post-Deployment
- [ ] Monitor error rate for 15 minutes
- [ ] Monitor latency for 15 minutes
- [ ] Test critical flows: login, send message, receive message
- [ ] Check logs for unexpected errors
- [ ] Confirm WebSocket connections re-establish

## 6. Rollback Procedure

### Backend Rollback
```bash
# Option A: Systemd
sudo systemctl stop ybm-connect
cp /opt/ybm-connect/bin/ybm-connect.previous /opt/ybm-connect/bin/ybm-connect
sudo systemctl start ybm-connect

# Option B: Docker
docker stop ybm-connect
docker run -d --name ybm-connect ybm-connect:{previous_version}
```

### Database Rollback
```bash
# If migration has DOWN script:
DATABASE_URL="..." sqlx migrate revert --source backend/migrations

# If migration cannot be reverted (data-destructive):
# Restore from backup
psql -h localhost -U ybm -d ybm_connect < backup_YYYYMMDD_HHMMSS.sql
```

### Frontend Rollback
- Cloudflare Pages: Use the dashboard to roll back to the previous deployment
- Manual: Re-deploy the previous build artifacts

## 7. Environment Configurations

| Environment | Backend | Database | Redis | Frontend | Purpose |
|-------------|---------|----------|-------|----------|---------|
| **Local** | `cargo run` | Docker (localhost:5432) | Docker (localhost:6379) | `npm run dev` | Development |
| **Staging** | VPS / Container | Managed or VPS | VPS | Cloudflare Pages (preview) | Pre-production testing |
| **Production** | VPS / Container | Managed or VPS | VPS | Cloudflare Pages (main) | Live users |

---

*Next: [CI-CD.md](CI-CD.md) · [infrastructure.md](infrastructure.md)*
