# terraform: reviewable, least-privilege infrastructure
**Done means these pass:** `terraform fmt -check -recursive`, `terraform init -backend=false && terraform validate`, `tflint`, a security scan (`checkov` or `trivy config`), and `terraform plan` shows exactly the intended changes (OpenTofu: `tofu` in place of `terraform`).
## Rules
1. Pin versions: `required_version`, `required_providers` with `~>` constraints, and commit `.terraform.lock.hcl`. Pin module sources to a version or commit.
2. Remote state with locking and encryption (S3 + DynamoDB/native locking, GCS, Terraform Cloud); never commit state or `*.tfvars` with secrets; mark variables `sensitive = true`; pull secrets from a secret manager, not variables; remember secrets still land in state.
3. Structure: small root modules per environment/blast radius, reusable child modules with typed `variable`s (`type`, `description`, `validation`) and documented `output`s; use `locals` for derived values; consistent naming and tagging (`default_tags`).
4. Prefer `for_each` over `count` for resources that have identity (adding or removing an item must not recreate the others); use `lifecycle { prevent_destroy = true }` on stateful resources and `create_before_destroy` where needed; `ignore_changes` sparingly and commented.
5. Least privilege IAM: no `*:*`, scope resources, use roles rather than long-lived keys; encryption at rest on storage and databases; private networking by default; security groups with specific CIDRs and ports, never `0.0.0.0/0` for admin ports.
6. Read the plan, never apply blindly: review destroys and replacements (`-/+`); use `terraform plan -out` and apply that exact plan in CI; no manual changes outside Terraform (detect drift with `plan -refresh-only`).
7. Use data sources rather than hard-coded ids; `moved` blocks and `import` blocks to refactor without destroying; `terraform state` surgery only with a backup.
8. Keep provisioners (`local-exec`, `remote-exec`) as a last resort; use `depends_on` only when an implicit dependency does not exist; avoid `null_resource` tricks.
9. Tests: `terraform test` (native tests), `terratest` for modules, policy checks (OPA/Sentinel/Conftest) in CI.
## Pitfalls
Renaming a resource destroys it unless moved; `count` index shifts; sensitive values in outputs and logs; provider upgrades changing behaviour; unversioned module sources; timeouts on slow resources.
## Versions
Pinned: Terraform 1.15 (1.15.8 (Jul 2026); 1.16 is in alpha; OpenTofu versions differ; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least Terraform 1.6 (native tests).
Rule: first run `terraform version` and read the project's pin (required_version, .terraform-version, .terraform.lock.hcl). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Terraform documentation and style guide, HashiCorp recommended practices, the CIS cloud benchmarks.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/terraform.md).

Canonical source: `plugins/web-development/middleware/languages/terraform/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/terraform.md` (reference; do not author).
