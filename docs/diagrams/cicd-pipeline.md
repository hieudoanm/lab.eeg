# CI/CD Pipeline

This diagram distinguishes the GitHub Actions validation workflow from the
Cloud Build deployment configuration. GitHub Actions runs on pull requests and
pushes to `main`; it validates but does not deploy. Cloud Build builds and
publishes images, runs backend/frontend tests, applies Terraform, and runs
migrations plus smoke tests for the dev path.

```mermaid
flowchart LR
    Change[Pull request or push]

    subgraph GitHub["GitHub Actions: .github/workflows/ci.yml"]
        Checkout[Checkout source]
        Backend[Backend checks\nRuff, mypy, migrations, pytest]
        Frontend[Frontend checks\nESLint, Prettier, typecheck, Vitest, build]
        E2E[Playwright smoke tests]
        Security[Trivy filesystem scan\nSARIF upload]
        Audits[Dependency audit artifacts]
        Checks[CI result]
    end

    subgraph Build["Cloud Build: infrastructure/cloudbuild.yaml"]
        BuildImages[Build API, worker,\nfrontend, dispatcher images]
        Tests[Backend and frontend tests]
        Push[Push commit-SHA and latest tags\nto container registry]
        DevPlan[Terraform dev plan]
        ProdPlan[Terraform prod plan]
        DevApply[Terraform dev apply]
        ProdApply[Terraform prod apply]
        Migration[Run SQL migrations on dev]
        Smoke[Run deployed dev E2E smoke test]
    end

    subgraph GCP["Google Cloud"]
        Registry[(Container registry)]
        Dev[Dev Cloud Run and managed services]
        Prod[Prod Cloud Run and managed services]
        Secrets[Secret Manager]
        State[(Terraform remote state)]
    end

    Change --> Checkout
    Change --> BuildImages
    Checkout --> Backend
    Checkout --> Frontend
    Checkout --> E2E
    Checkout --> Security
    Backend --> Audits
    Frontend --> Audits
    Backend --> Checks
    Frontend --> Checks
    E2E --> Checks
    Security --> Checks

    BuildImages --> Push
    Tests --> DevPlan
    Tests --> ProdPlan
    Push --> DevPlan
    Push --> ProdPlan
    BuildImages --> Tests
    Push --> Registry
    DevPlan --> DevApply
    ProdPlan --> ProdApply
    DevApply --> Migration
    Migration --> Smoke

    Secrets -.->|Terraform plan inputs| DevPlan
    Secrets -.->|Terraform plan inputs| ProdPlan
    State <--> DevPlan
    State <--> ProdPlan
    DevApply --> Dev
    ProdApply --> Prod
    Registry --> Dev
    Registry --> Prod
```

**Configuration caveats:** the Cloud Build YAML comments describe intended
trigger routing (`main` to dev and `v*` tags to prod), but this file contains no
branch/tag conditions; those must be enforced by the configured Cloud Build
triggers. Both dev and prod plan/apply steps are present in this YAML. The
configured release sequence runs migrations and smoke tests only for dev; it
does not define a production migration or production smoke-test step. The
GitHub Actions workflow and Cloud Build are separate configurations; this
diagram does not imply that a successful GitHub Actions run automatically
starts Cloud Build.
