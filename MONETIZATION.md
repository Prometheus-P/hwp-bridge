# Monetization Playbook (for HwpBridge)

You said you want **reputation first**, but also a clear path to revenue.
Here’s the cleanest path that doesn't ruin adoption.

## 1) Reputation engine (free, loud, measurable)
- Publish **benchmarks**: speed, fidelity metrics, crash rate, encrypted-file detection.
- Ship a **Smithery MCP** so people can try it in Claude Desktop in 30 seconds.
- Maintain a public **format coverage matrix** (what you parse perfectly vs partially).

Deliverables to keep updating:
- `docs/BENCHMARKS.md`
- `docs/FORMAT_SUPPORT.md`
- `docs/SECURITY.md`

## 2) Revenue that doesn't kill open-source
### A. Open-core (recommended)
- Core stays Apache-2.0.
- Sell **Pro add-ons** as a separate repo/package under a commercial license.

### B. Paid support / SLA
- “We keep your pipeline running” money.
- Works well for enterprises that hate risk.

### C. Hosted API
- You host conversion. Charge by documents/month.
- Best margin, but you now own infra + compliance.

## 3) Don't do this (it backfires)
- “Business use requires payment” on an Apache/MIT repo.
  - It’s not enforceable unless you change the license.
  - Adoption collapses, and your reputation takes a hit.

## 4) Pricing anchors (starter numbers)
- Pro add-on license: $299–$999/month per company (usage-based), or $5k–$50k/year
- SLA/support: $2k–$15k/month depending on response time
- Hosted API: $0.01–$0.05 per page equivalent (or per doc tiers)

## 5) The real wedge (why people pay)
They pay for **fidelity guarantees**, **compliance**, and **time-to-integrate**.
Make those measurable:
- Fidelity score regression (golden fixtures)
- Deterministic outputs
- Audit logs + retention controls
