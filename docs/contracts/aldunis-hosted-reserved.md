# Reserved: Aldunis hosted mode

Status: reserved, unimplemented.

QuantForge may later run as an enterprise-managed hosted product behind Aldunis,
following the Code/Workshop assertion pattern.

## Reserved contract

- Browser talks only to Aldunis (session cookie).
- Caddy forward-auth mints a short-lived EdDSA assertion.
- The assertion is copied only to the private QuantForge upstream.
- QuantForge verifies issuer, audience, signature, expiry, tenant, instance,
  and scopes, then applies its own domain authorization.
- Caller-selected tenant headers never establish authority.
- QuantForge keeps its own PostgreSQL or SQLite. It never reads the Aldunis
  database and never receives platform credentials.

## Not in this repository

Catalog adapters, hosted-product grants, Tenkai rollout env, and Caddy routes
belong in private `aldunis-platform`. This repo publishes source and, later,
digest-pinned images only.

`make start` must not mention or require Aldunis.
