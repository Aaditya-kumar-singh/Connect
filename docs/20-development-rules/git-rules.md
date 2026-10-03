# Git Workflow — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-DEV-003`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |

---

## Branch Strategy

```
main ──────────────────────────────────────── Production-ready
  │
  └── develop ─────────────────────────────── Integration branch
        │
        ├── feature/auth-registration ─────── New feature
        ├── feature/message-reactions ──────── New feature
        ├── fix/websocket-reconnect ────────── Bug fix
        ├── hotfix/token-validation ────────── Urgent production fix
        ├── refactor/messaging-service ─────── Code improvement
        └── docs/api-documentation ─────────── Documentation only
```

| Branch Pattern | Source | Merges Into | Purpose |
|---------------|--------|-------------|---------|
| `main` | — | — | Production. Always deployable. |
| `develop` | `main` | `main` | Integration. Features merge here first. |
| `feature/*` | `develop` | `develop` | New features. Named: `feature/{scope}-{description}` |
| `fix/*` | `develop` | `develop` | Bug fixes. Named: `fix/{scope}-{description}` |
| `hotfix/*` | `main` | `main` + `develop` | Urgent production fixes. |
| `refactor/*` | `develop` | `develop` | Code improvements without feature changes. |
| `docs/*` | `develop` | `develop` | Documentation-only changes. |

## Commit Convention

Format: `{type}({scope}): {description}`

| Type | When to Use | Example |
|------|-------------|---------|
| `feat` | New feature | `feat(messaging): add message reactions` |
| `fix` | Bug fix | `fix(websocket): handle reconnect on token expiry` |
| `refactor` | Code change that doesn't add features or fix bugs | `refactor(auth): extract token rotation logic` |
| `perf` | Performance improvement | `perf(db): add index on messages.conversation_id` |
| `docs` | Documentation only | `docs(api): document media upload endpoint` |
| `test` | Adding or updating tests | `test(messaging): add duplicate send test` |
| `build` | Build system or dependencies | `build(cargo): update sqlx to 0.8` |
| `ci` | CI/CD changes | `ci: add integration test stage` |
| `chore` | Maintenance tasks | `chore: update .gitignore` |
| `security` | Security-related change | `security(auth): add rate limiting to login` |

**Rules:**
- Subject line: imperative mood, lowercase, no period, max 72 chars
- Body: explain WHY, not just WHAT (the diff shows what changed)
- Breaking changes: add `BREAKING CHANGE:` footer

## Pull Request Template

```markdown
## Problem
What issue does this PR address? Link to issue/requirement if applicable.

## Solution
How does this PR solve the problem? Explain the approach.

## Files Changed
- `src/messaging/service.rs` — Added reaction logic
- `src/messaging/repository.rs` — Added reaction queries
- `migrations/015_message_reactions.sql` — New table

## Architecture Impact
- [ ] No architecture changes
- [ ] New module/service added: _________
- [ ] Module boundary changed: _________
- [ ] New external dependency: _________

## Database Changes
- [ ] No database changes
- [ ] New migration added (reversible: yes/no)
- [ ] Migration tested (up and down)

## API Changes
- [ ] No API changes
- [ ] New endpoint: _________
- [ ] Changed endpoint: _________
- [ ] Breaking change (version bump needed): _________

## Security Impact
- [ ] No security impact
- [ ] Authentication change: _________
- [ ] Authorization change: _________
- [ ] Input validation change: _________
- [ ] New attack surface: _________

## Tests
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] All existing tests pass

## Deployment Impact
- [ ] No deployment changes needed
- [ ] Database migration required
- [ ] Environment variable added: _________
- [ ] Infrastructure change: _________

## Rollback Plan
How to revert this change if something goes wrong in production:
_________
```

## Code Review Checklist

- [ ] Code follows the layered architecture (Handler → Service → Repository)
- [ ] No business logic in handlers
- [ ] No direct database access outside repository layer
- [ ] Error handling is complete (no unwrap in production code)
- [ ] Input validation on all user input
- [ ] Authorization check on all endpoints
- [ ] No secrets hardcoded
- [ ] No sensitive data logged
- [ ] Tests cover happy path and error paths
- [ ] Documentation updated if API or architecture changed
- [ ] Database migration is reversible
- [ ] No unnecessary dependencies added

---

*Next: [branch-strategy.md](branch-strategy.md) · [commit-conventions.md](commit-conventions.md)*
