# 0001: QuantForge product boundary

- Status: Accepted
- Date: 2026-08-15

## Context

QuantForge is a public MIT research workbench. Aldunis Platform is a private
enterprise gateway. Sekai Chisei is a public governance control plane. A
fundamentals product has its own data plane and cannot be a Workshop CRUD
archetype or an Aldunis console module.

## Decision

QuantForge is an independently versioned local-first product.

- The host binds loopback only and owns SQLite at `~/.quantforge/quantforge.db`.
- The browser never receives provider keys, database paths, or platform credentials.
- Sekai Chisei is an optional later projection plane. It is not the research store
  and is not required to start. See [chisei-projection-reserved](../contracts/chisei-projection-reserved.md).
- Aldunis hosted mode is a later assertion/gateway seam. See
  [aldunis-hosted-reserved](../contracts/aldunis-hosted-reserved.md).
- QuantForge does not vendor or publish `aldunis-platform` or `aldunis-workshop`.

## Consequences

`make start` works without Keycloak, Aldunis, Chisei, or a paid data key. The
fixture provider is the offline default. Yahoo is unofficial. FMP needs a
host-side key.
