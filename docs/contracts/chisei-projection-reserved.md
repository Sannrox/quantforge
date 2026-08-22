# Reserved: Sekai Chisei projections

Status: reserved, unimplemented.

QuantForge may later show Chisei projections the way Aldunis Code does:
server-owned, fail-visible, never treated as domain authority.

## Reserved host configuration

| Name | Meaning |
| --- | --- |
| `CHISEI_ENDPOINT` | gRPC endpoint, typically `http://127.0.0.1:50051` |
| `CHISEI_TOKEN` | Optional bearer. Never returned to the browser. |
| `CHISEI_NAMESPACE` | Namespace binding stored by the host, not chosen per request |

Absence of this configuration is normal. The research UI must not mention
Chisei unless the host later exposes a projection descriptor.

## Later ownership (not built)

- Lineage of a provider fetch (source, time, adapter, digest) as evidence
- Policy on egress to Yahoo or FMP
- Audit of refresh and DCF assumption changes
- Optional later AI routing via `PlanExecution`

## Forbidden

- Requiring a running Chisei process
- Persisting statements into Sekai as the primary store
- Treating a projection as quote, multiple, or DCF authority
- Sending provider keys or raw vendor payloads to the browser or to Chisei

Reserved ontology class names for a later document: `Issuer`,
`FinancialPeriod`, `ValuationAssumption`.
