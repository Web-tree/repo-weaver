# 20 - Kubernetes Ingress Annotation Policy (`rw check`)

Demonstrates policy validation for ingress manifests: every ingress must include
required controller and ownership annotations.

## What this covers

- `rw check [app]` command behavior (PRD §8)
- Realistic policy guardrail for Kubernetes ingress standards
- Non-mutating validation workflow for pre-merge checks

## Policy intent

All ingress resources must declare:
- `kubernetes.io/ingress.class`
- `external-dns.alpha.kubernetes.io/hostname`
- `platform.company.io/owner`

## How to run

```sh
cd before
rw check gateway
```

## Expected result

`rw check gateway` should fail in `before/` because required annotations are
missing.

`after/` shows the expected compliant state once the ingress is corrected.
