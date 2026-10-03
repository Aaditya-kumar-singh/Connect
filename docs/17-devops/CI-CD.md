# CI/CD Pipeline — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-DEVOPS-003`                           |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |

---

## 1. Pipeline Overview

```mermaid
flowchart LR
    A[Push/PR] --> B[Lint]
    B --> C[Unit Tests]
    C --> D[Build]
    D --> E[Integration Tests]
    E --> F{Branch?}
    F -->|develop| G[Deploy Staging]
    F -->|main| H[Deploy Production]
    F -->|feature/*| I[Report Results]
```

## 2. GitHub Actions Workflow

### `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  DATABASE_URL: postgres://ybm:ybm_test@localhost:5432/ybm_connect_test
  REDIS_URL: redis://localhost:6379

jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings
      - name: Frontend lint
        run: |
          cd frontend/web
          npm ci
          npx eslint .

  test-unit:
    name: Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run unit tests
        run: cargo test --lib --no-fail-fast

  test-integration:
    name: Integration Tests
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_USER: ybm
          POSTGRES_PASSWORD: ybm_test
          POSTGRES_DB: ybm_connect_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install sqlx-cli
        run: cargo install sqlx-cli --features postgres --no-default-features
      - name: Run migrations
        run: sqlx migrate run --source backend/migrations
      - name: Run integration tests
        run: cargo test --test integration --no-fail-fast

  build:
    name: Build
    runs-on: ubuntu-latest
    needs: [lint, test-unit]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Build release
        run: cargo build --release
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: backend-binary
          path: target/release/ybm-connect

  build-frontend:
    name: Build Frontend
    runs-on: ubuntu-latest
    needs: [lint]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
          cache-dependency-path: frontend/web/package-lock.json
      - name: Install dependencies
        run: cd frontend/web && npm ci
      - name: Build
        run: cd frontend/web && npm run build
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: frontend-build
          path: frontend/web/out

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-audit
        run: cargo install cargo-audit
      - name: Run audit
        run: cargo audit
      - name: NPM audit
        run: cd frontend/web && npm audit --production
```

### `.github/workflows/deploy.yml`

```yaml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    name: Deploy to Production
    runs-on: ubuntu-latest
    needs: [ci]  # Requires CI workflow to pass
    environment: production
    steps:
      - uses: actions/checkout@v4
      
      - name: Download backend binary
        uses: actions/download-artifact@v4
        with:
          name: backend-binary
      
      - name: Run database migrations
        run: |
          sqlx migrate run --source backend/migrations
        env:
          DATABASE_URL: ${{ secrets.PRODUCTION_DATABASE_URL }}
      
      - name: Deploy backend
        run: |
          scp ybm-connect ${{ secrets.DEPLOY_USER }}@${{ secrets.DEPLOY_HOST }}:/opt/ybm-connect/bin/
          ssh ${{ secrets.DEPLOY_USER }}@${{ secrets.DEPLOY_HOST }} "sudo systemctl restart ybm-connect"
      
      - name: Verify deployment
        run: |
          sleep 10
          curl -f https://api.ybmconnect.com/health || exit 1
          curl -f https://api.ybmconnect.com/ready || exit 1
      
      - name: Deploy frontend
        uses: cloudflare/wrangler-action@v3
        with:
          command: pages deploy frontend/web/out --project-name ybm-connect
          apiToken: ${{ secrets.CLOUDFLARE_API_TOKEN }}
```

## 3. Pipeline Stages

| Stage | Trigger | Duration | Failure Action |
|-------|---------|----------|----------------|
| **Lint** | Every push/PR | ~1 min | Block merge |
| **Unit Tests** | Every push/PR | ~2 min | Block merge |
| **Build** | After lint + unit tests | ~5 min | Block merge |
| **Integration Tests** | Every push/PR | ~5 min | Block merge |
| **Security Audit** | Every push/PR | ~2 min | Warn (don't block) |
| **Deploy Staging** | Push to `develop` | ~3 min | Alert team |
| **Deploy Production** | Push to `main` | ~5 min | Alert team, auto-rollback |

## 4. Branch Protection Rules

### `main` branch
- Require pull request before merging
- Require status checks: `lint`, `test-unit`, `test-integration`, `build`
- Require conversation resolution
- No force pushes
- No deletions

### `develop` branch
- Require pull request before merging
- Require status checks: `lint`, `test-unit`
- No force pushes

---

*Next: [deployment.md](deployment.md) · [monitoring.md](monitoring.md)*
