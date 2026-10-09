# EEG Lab — Project Overview

## 1. Project Summary

**EEG Lab** is a full-stack scientific SaaS application for neuroscience researchers to organise studies, upload EEG recordings, explore neural signals, run asynchronous analyses, and share results securely with collaborators.

The project demonstrates end-to-end software engineering across a React and TypeScript frontend, asynchronous Python backend, PostgreSQL database, background processing, object storage, cloud infrastructure, and automated deployment.

EEG Lab is an independent portfolio project inspired by the engineering challenges of scientific software. It must not copy proprietary software, branding, source code, or confidential workflows from any existing company.

## 2. Problem Statement

Neuroscience research often involves multiple studies, large signal recordings, metadata, computational analyses, and collaboration between researchers.

A useful research platform needs to make it possible to:

- Organise research studies and associated datasets.
- Upload recordings and validate their structure.
- Inspect signals through interactive scientific visualisations.
- Execute analyses without blocking the web application.
- Track processing status, failures, and results.
- Preserve analysis parameters and data provenance.
- Control access to research data across organisations.
- Maintain reliable, secure, and reproducible infrastructure.

EEG Lab brings these workflows together in one coherent application.

## 3. Product Vision

Build a polished, production-style scientific SaaS platform that demonstrates how modern full-stack engineering can support neuroscience research.

The product should be sufficiently complete that a reviewer can create a study, upload or select an EEG recording, inspect the signal, launch an analysis, and examine its results without manually interacting with the database or backend.

The project should prioritise engineering quality, usability, security, and reproducibility over the number of features.

## 4. Target Users

### 4.1 Researcher

A neuroscience researcher who wants to organise studies, inspect EEG recordings, run analyses, and review results.

Primary needs:

- Create and manage studies.
- Upload datasets and inspect their metadata.
- Explore channels and time intervals.
- Submit and monitor analysis jobs.
- Review results and their provenance.

### 4.2 Viewer

A collaborator who needs read-only access to permitted research resources.

Primary needs:

- View studies and datasets.
- Inspect available signal visualisations.
- Read analysis results.
- Understand when data was collected or processed.

### 4.3 Organisation Owner

A person responsible for a research workspace and its members.

Primary needs:

- Manage organisation membership.
- Assign roles.
- Protect organisation data.
- Review relevant activity.

### 4.4 Developer or Operator

A person responsible for maintaining and deploying EEG Lab.

Primary needs:

- Run the application locally.
- Diagnose failed requests and analysis jobs.
- Apply database migrations safely.
- Deploy and roll back application releases.
- Monitor application health and infrastructure costs.

## 5. Product Goals

### Goal 1: Deliver a complete scientific workflow

A user can move from study creation to dataset exploration and analysis results within the web application.

### Goal 2: Demonstrate full-stack ownership

The implementation covers frontend development, API design, database modelling, background processing, infrastructure, testing, and deployment.

### Goal 3: Make scientific data understandable

The application presents signal data, metadata, analysis parameters, and results in a clear and accessible interface.

### Goal 4: Establish secure multi-tenancy

Different organisations can use the application without accessing one another's studies, datasets, jobs, or results.

### Goal 5: Support reproducible analysis

Each analysis records its algorithm version, parameters, input dataset version, and execution status.

### Goal 6: Provide a maintainable codebase

The repository has clear architectural boundaries, automated tests, reproducible setup instructions, and documented engineering decisions.

## 6. Non-Goals

The initial release will not:

- Diagnose neurological conditions.
- Recommend medical treatment.
- Claim clinical validity or regulatory certification.
- Acquire EEG data directly from physical hardware.
- Implement real-time brain-computer interface control.
- Decode imagined speech or claim to interpret thoughts.
- Train large-scale neural networks.
- Replace specialist neuroscience analysis software.
- Support every scientific file format.
- Implement billing, subscriptions, or a commercial pricing system.
- Use microservices or Kubernetes without a demonstrated need.

These features may be considered in future iterations, but they must not compromise the initial delivery.

## 7. Scientific Data

The initial demonstration will use a small subset of the public PhysioNet EEG Motor Movement/Imagery Dataset.

Dataset: https://physionet.org/content/eegmmidb/

The dataset provides EEG recordings associated with motor movement and motor imagery experiments.

The application should preserve available provenance information, including:

- Dataset source and source URL.
- Recording identifier.
- Sampling frequency.
- Channel names.
- Number of channels and samples.
- Duration and measurement units, when known.
- File checksum.
- Validation status.
- Preprocessing and analysis parameters.

The project must respect the dataset's published terms and citation requirements.

Large raw datasets must not be committed to the Git repository. Use small synthetic fixtures for automated tests and development, and download public sample data through documented instructions when needed.

Only public or synthetic data should be used in the portfolio demonstration.

## 8. Primary User Workflow

The primary workflow is:

1. A researcher signs in.
2. The researcher selects an organisation.
3. The researcher creates or opens a study.
4. The researcher uploads an EEG recording and supplies its metadata.
5. EEG Lab validates the file and records its provenance.
6. The researcher opens the dataset explorer.
7. The researcher selects channels and a time interval.
8. The researcher launches a descriptive analysis.
9. EEG Lab processes the analysis asynchronously.
10. The researcher reviews the job status and results.
11. The researcher can revisit the results and inspect the recorded parameters.

Every stage should provide meaningful feedback, including loading states, validation errors, processing progress, and recovery instructions.

## 9. Technology Direction

The intended technology stack is:

- **Frontend:** React and TypeScript.
- **Scientific visualisation:** Plotly.js.
- **Backend API:** Python, FastAPI, and Pydantic.
- **Database access:** PostgreSQL using asynchronous drivers and explicit SQL.
- **Background processing:** A durable queue and separate worker.
- **File storage:** Object storage for recordings and large outputs.
- **Infrastructure:** Google Cloud Platform and Terraform.
- **CI/CD:** GitHub Actions.
- **Testing:** Frontend unit and browser tests, backend unit and integration tests, and security-focused tests.

The detailed decisions and rationale are specified in `08-tech-stack.md`.

Local commits use Husky and lint-staged to apply fast formatting and lint checks to staged backend and frontend source files. CI remains the authoritative full quality gate.

## 10. Security and Privacy Principles

Security must be implemented as part of the architecture, not added as a final polishing step.

The application must:

- Authenticate users before granting access to private resources.
- Enforce authorisation on the server.
- Validate organisation membership for every organisation-scoped operation.
- Prevent cross-tenant access to records and stored files.
- Use parameterised SQL.
- Keep secrets out of source control and frontend bundles.
- Limit upload sizes and validate file contents.
- Use private object storage and short-lived signed URLs where appropriate.
- Avoid logging credentials, access tokens, or raw signal data unnecessarily.
- Provide automated tests for unauthorised access attempts.

EEG Lab is a portfolio demonstration and must not claim compliance with HIPAA, ISO 27001, medical-device regulations, or other certifications without independently establishing the applicable requirements and evidence.

## 11. MVP Definition of Done

The minimum viable product is complete when:

- A developer can run the application using documented instructions.
- A user can access a workspace and create a study.
- A supported EEG recording can be uploaded or loaded from a documented sample.
- Dataset metadata and provenance are visible.
- A bounded signal interval can be explored through an interactive chart.
- A descriptive analysis can be submitted as a background job.
- Job status and results are available in the frontend.
- Invalid data and failed jobs produce understandable errors.
- Organisation and role permissions are enforced.
- Automated tests cover the core workflow and tenant isolation.
- CI runs the required quality checks.
- The architecture and operational limitations are documented.
- A cloud deployment can be reproduced using Terraform when cloud credentials are available.

## 12. Success Criteria

The project should demonstrate the following outcomes:

| Area                 | Success criterion                                                                 |
| -------------------- | --------------------------------------------------------------------------------- |
| User experience      | A reviewer can complete the primary workflow without developer assistance.        |
| Frontend             | Strictly typed React components with meaningful loading, error, and empty states. |
| Backend              | Documented REST endpoints with validated request and response models.             |
| Database             | Versioned migrations, parameterised SQL, and appropriate indexes.                 |
| Scientific computing | Correct descriptive statistics verified against known test data.                  |
| Background jobs      | Durable job states, bounded retries, and idempotent processing.                   |
| Security             | Automated negative tests for cross-tenant access.                                 |
| Testing              | Repeatable automated checks covering critical workflows.                          |
| DevOps               | Reproducible infrastructure and documented deployment procedures.                 |
| Portfolio            | A clear README, architecture diagram, demo, and documented trade-offs.            |

## 13. Guiding Principles

1. **Working software first:** deliver a complete vertical slice before expanding the scope.
2. **Scientific integrity:** preserve provenance and distinguish display transformations from analytical computations.
3. **Secure by default:** deny access unless authentication and authorisation checks succeed.
4. **Explicit contracts:** use documented API models and database migrations.
5. **Reliable asynchronous processing:** assume jobs may be delivered more than once or fail midway.
6. **Simple architecture:** prefer a modular monolith and separate worker over unnecessary distributed complexity.
7. **Reproducibility:** record the inputs, parameters, and versions needed to understand an analysis.
8. **Evidence over claims:** document only features that have actually been implemented and tested.
9. **Incremental delivery:** keep every implementation milestone runnable and reviewable.
10. **No clinical claims:** present EEG Lab as a research-software demonstration, not a medical product.
