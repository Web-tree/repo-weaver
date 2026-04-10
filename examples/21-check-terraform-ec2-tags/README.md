# 21 - Terraform EC2 Required Tags Policy (`rw check`)

Demonstrates policy validation for Terraform-managed EC2 instances where all
instances must carry mandatory governance tags.

## What this covers

- `rw check [app]` for infrastructure policy validation (PRD §8)
- Realistic cloud-governance scenario (required owner/env/cost-center tags)
- Non-mutating pre-apply checks in CI pipelines

## Policy intent

Every `aws_instance` resource must include:
- `Owner`
- `Environment`
- `CostCenter`

## How to run

```sh
cd before
rw check compute
```

## Expected result

`rw check compute` should fail in `before/` because required tags are missing.

`after/` shows the expected compliant Terraform state.
