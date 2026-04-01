# Catalog Service Runbook

## Readiness checklist

- Confirm `/healthz` returns `200` in the deployment environment.
- Verify database migrations are up to date.
- Validate that service logs include request IDs.

## On-call quick actions

- Restart deployment if memory exceeds baseline for 10 minutes.
- Roll back to the previous image tag when startup checks fail.
