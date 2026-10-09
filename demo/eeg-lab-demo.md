# EEG Lab — Feature Demo Script

A guided, scene-by-scene script that exercises **every user-visible feature** of
EEG Lab: multi-tenant sign-in, organisation context, study and dataset
management, secure EEG upload and validation, interactive signal exploration,
asynchronous descriptive analysis with provenance, organisation administration,
tenant isolation, and operational health.

- **Audience:** reviewers, interviewers, or a recorded portfolio walkthrough.
- **Runtime:** ~10–12 minutes for the full script.
- **Data:** deterministic synthetic recordings only — no real participant data.
- **Claim discipline:** everything below is implemented and tested. Descriptive
  statistics are _not_ clinical findings.

---

## 1. What this demo proves

| #   | Scene                | Feature / capability                                              | User story             |
| --- | -------------------- | ----------------------------------------------------------------- | ---------------------- |
| 0   | Environment          | Health, readiness, metrics                                        | US-021                 |
| 1   | Sign in              | Passwordless development identity, HttpOnly session cookie        | US-001                 |
| 2   | Dashboard            | Organisation-scoped counts and recent activity                    | US-002                 |
| 3   | Create study         | Study lifecycle (create)                                          | US-005                 |
| 4   | Browse studies       | Search, status filter, cursor pagination, empty/no-results states | US-005                 |
| 5   | Study detail         | Metadata, dataset list, edit, archive                             | US-006                 |
| 6   | Register dataset     | Dataset registration in a study                                   | US-007                 |
| 7   | Upload & validate    | Upload session → content → synchronous validation → `ready`       | US-007, US-008         |
| 8   | Signal explorer      | Channel selection, time windows, downsampling, accessible summary | US-010, US-011, US-012 |
| 9   | Submit analysis      | `Idempotency-Key` deduplication, asynchronous job                 | US-013                 |
| 10  | Monitor & cancel     | Status badges, org-wide list, filter, cooperative cancel          | US-014, US-018         |
| 11  | Results & provenance | Per-channel statistics, checksums, algorithm version              | US-015, US-017         |
| 12  | Settings             | Organisation members, roles, profile, memberships                 | US-019, US-003         |
| 13  | Tenant isolation     | Cross-organisation access is rejected                             | US-002                 |

---

## 2. Prerequisites

- Docker and Docker Compose
- `pnpm` / Node.js (only for the automated recording option)
- `curl` (for the optional API tour)
- ~4 GB free disk for images and volumes

The demo recording fixture is committed:

```text
demo/fixtures/resting-state-eeg.npy   4 channels x 4096 samples, float32, 16 s @ 256 Hz
demo/fixtures/resting-state-eeg.json  matching manifest
```

---

## 3. Option A — Automated recording (one command)

This starts the full stack, drives a headless Chromium through the workflow, and
renders `demo/eeg-lab-demo.mp4`. It requires `docker`, `pnpm`, `ffmpeg`, and
`ffprobe`.

```bash
./scripts/create_demo.sh
```

The Compose services **stay running** afterwards. Stop them with:

```bash
docker compose -p eeg-lab down
```

For a narrated walkthrough, continue with Option B.

---

## 4. Option B — Guided walkthrough

### 4.1 Start the stack

```bash
# 1. Start PostgreSQL, Redis, API, worker, dispatcher and the frontend
docker compose up --build -d

# 2. Wait until the API reports ready
curl --fail http://localhost:8000/health/ready
```

Expected: a JSON body with `"status": "ready"` and a healthy database.

| Service            | URL                           |
| ------------------ | ----------------------------- |
| Frontend           | http://localhost:3000         |
| API                | http://localhost:8000/api/v1  |
| API docs (Swagger) | http://localhost:8000/docs    |
| Metrics            | http://localhost:8000/metrics |

### 4.2 Cast

- **Demo researcher** — signs in with any email (passwordless dev identity).
- **Fixture recording** — `demo/fixtures/resting-state-eeg.npy`
  (four channels, `EEG 001`–`EEG 004`, in microvolts).

### Scene index

| Scene | Title                                 |
| ----- | ------------------------------------- |
| 0     | Operational health                    |
| 1     | Sign in                               |
| 2     | Dashboard and organisation context    |
| 3     | Create a study                        |
| 4     | Browse, search and filter studies     |
| 5     | Study detail: datasets, edit, archive |
| 6     | Register a dataset                    |
| 7     | Upload and validate a recording       |
| 8     | Signal exploration                    |
| 9     | Submit a descriptive analysis         |
| 10    | Monitor analyses and cancel           |
| 11    | Results and provenance                |
| 12    | Settings: organisation and profile    |
| 13    | Tenant isolation                      |

---

### Scene 0 — Operational health

**Narration:** "EEG Lab is a durable, multi-tenant service. Before anything
else, I confirm the API process is alive and its dependencies are ready."

**Do**

```bash
curl -s http://localhost:8000/health/live    # process liveness
curl -s http://localhost:8000/health/ready   # database dependency
curl -s http://localhost:8000/metrics | head # Prometheus counters/histograms
```

**Expected:** `health/live` returns service, version and environment;
`health/ready` returns `status: ready` with a healthy database; `/metrics`
exposes HTTP, job, outbox, storage and database metrics.

**Talking point:** health endpoints never reveal secrets or connection strings;
`/healthz` and `/readyz` are undocumented aliases.

---

### Scene 1 — Sign in

**Narration:** "Access is deny-by-default. The browser authenticates through the
Next.js BFF, which keeps the bearer token in an HttpOnly session cookie so it is
never exposed to JavaScript."

**Do**

1. Open http://localhost:3000.
2. Under **Email address**, enter `demo-researcher@eeg-lab.local`.
3. Click **Sign in**.

**Expected:** redirected to `/app/dashboard`; a session cookie
(`eeg-lab_session`) is set. Invalid responses are described safely and no other
accounts are disclosed.

**Talking point:** `POST /auth/token` is development-only; it returns `404`
unless `AUTH_MODE=development`, and is never available in production.

---

### Scene 2 — Dashboard and organisation context

**Narration:** "Everything is scoped to an organisation. The dashboard summarises
the active organisation."

**Do**

1. Observe the **Dashboard** heading and the summary cards:
   **Studies** (with active count), **Datasets** (with ready count), and
   **Analyses** (with succeeded / failed counts).
2. Look at **Recent studies** and **Recent analyses**.
3. If your user belongs to more than one organisation, use the
   **Organisation** switcher in the top bar to change scope.

**Expected:** counts and recent items reflect only the active organisation;
switching organisations changes the visible data. With no membership, the app
shows a "No organisation" empty state.

**Talking point:** the organisation id in a URL is a _selector_, not proof of
permission — the server re-verifies membership and role on every request.

---

### Scene 3 — Create a study

**Narration:** "A study is the container for datasets and analyses."

**Do**

1. In the left navigation (**Primary**), click **Studies**.
2. Under **New study**, fill **Name** = `EEG Lab Demo Study` and
   **Description** = `Synthetic EEG workflow demonstration`.
3. Click **Create study**.

**Expected:** navigated to the new study page at `/app/studies/<id>`; the study
name is the page title. Inline validation blocks empty or over-long names.

**Talking point:** the form prevents accidental duplicate submissions and shows
server errors without leaking internals.

---

### Scene 4 — Browse, search and filter studies

**Do**

1. Return to **Studies**.
2. In the filter form, set **Search** = `Demo`, **Status** = `Active`, then click
   **Apply**.
3. Clear the search and confirm the **All studies** view.

**Expected:** filtered results show a "Filtered results" card description; a
query with no matches shows a distinct no-results state; an empty organisation
shows a helpful empty state. **Next page** appears when more than one page
exists.

**Talking point:** pagination is cursor-based and deterministic, and cannot
expose resources outside the organisation.

---

### Scene 5 — Study detail: datasets, edit, archive

**Do**

1. Click the **EEG Lab Demo Study** link.
2. Observe the metadata grid (**Status**, **Created**, **Updated**, **Study ID**)
   and the **Datasets** table (**Name**, **Format**, **Status**, **Created**).
3. Optionally edit the study fields and click **Save changes**.
4. To archive, click **Archive study**, then **Confirm archive**.

**Expected:** archiving makes the study read-only — the dataset form, upload
controls and analysis actions disappear, the datasets table remains, and a note
explains that datasets, results and history are kept.

**Talking point:** archiving never silently deletes datasets, results, or audit
history.

---

### Scene 6 — Register a dataset

**Narration:** "Registering metadata and uploading the recording are separate
steps, so a dataset is never marked ready before validation."

**Do**

1. In **Register dataset**, fill:
   - **Name** = `Resting-state EEG recording`
   - **Description** = `Four-channel synthetic recording`
   - **Source** = `Synthetic data generated for this demonstration`
2. Click **Register dataset**.

**Expected:** navigated to
`/app/studies/<studyId>/datasets/<datasetId>`; **Status** = `Pending upload`.

**Talking point:** the client cannot choose a storage key or mark a dataset
ready; the backend owns that.

---

### Scene 7 — Upload and validate a recording

**Narration:** "The recording travels through a Next.js route handler to an
authenticated upload endpoint, then validation runs synchronously before the
dataset becomes ready."

**Do**

1. On the dataset page, under **Upload recording**:
   - **Recording file**: choose `demo/fixtures/resting-state-eeg.npy`.
   - **Sampling frequency (Hz)**: `256`.
   - **Channel names** (one per line): `EEG 001`, `EEG 002`, `EEG 003`,
     `EEG 004`.
   - **Channel units** (one per line): `uV`, `uV`, `uV`, `uV`.
2. Click **Upload recording**.
3. Watch the stepper: **Upload → Validation → Ready**, and the progress bar.

**Expected:** on success a toast reads "Recording ready", the dataset **Status**
becomes **Ready**, and the metadata grid fills in **Channels**, **Samples**,
**Duration**, **File size**, and the SHA-256 **Checksum**.

**Talking point:** invalid formats, oversized files, mismatched channel counts,
and corrupt `.npy` data are rejected _before_ the dataset is marked ready;
object data is never loaded with pickle enabled.

**Negative variant (optional):** upload a file with a wrong channel count and
show the safe validation error.

---

### Scene 8 — Signal exploration

**Narration:** "A ready recording becomes explorable."

**Do**

1. In the **Signal visualisation** card, use the **Channel** selector to switch
   channels.
2. Change the **Window (s)** start and end inputs (e.g. `2` to `6`); note the
   **Duration: Xs** readout.
3. Use **← Prev** and **Next →** to move between channels.
4. Read the **Channel data summary**: **Samples**, **Valid**, **Time range**,
   **Sampling**.

**Expected:** the chart redraws for the selected channel and window; a
"Display is downsampled by …" note appears when the window exceeds the point
budget; the visible data is also summarised as text/tables for accessibility.

**Talking point:** the browser requests only a bounded interval of one channel —
never the whole recording — and invalid windows or channel selections are
rejected server-side. Display downsampling never modifies the source recording.

---

### Scene 9 — Submit a descriptive analysis

**Narration:** "Analyses run asynchronously in a worker, coordinated by a
transactional outbox so accepted jobs are never lost."

**Do**

1. In the **Analyses** card on the dataset page, note the auto-generated,
   read-only idempotency key.
2. Click **Submit analysis**.
3. Watch the row appear with a status badge.

**Expected:** a new job appears with status **Queued**, then **Running**, then
**Succeeded** (within a few seconds on the local worker).

**Talking point:** submitting with the same idempotency key returns the _same_
job; retries are bounded by the durable job (`attempt_count` / `max_attempts`),
not Celery. See the literal idempotency check in the API tour (§5).

---

### Scene 10 — Monitor analyses and cancel

**Do**

1. Click **Analyses** in the navigation.
2. Use the **Status** filter and click **Apply** (try `Succeeded`, then
   `Cancelled`).
3. Back on the dataset page, click **Submit analysis** again, then immediately
   click **Cancel** on the queued row.

**Expected:** the analyses page shows an org-wide table (**Status**, **Type**,
**Dataset**, **Requested**) with cursor pagination (**Next page**) and a
no-results state for empty filters. The second job transitions to
**Cancelled**; cancelling again is idempotent. Only queued jobs can be cancelled.

**Talking point:** the org-wide list orders by `requested_at` descending then
`id`, so pagination is stable.

---

### Scene 11 — Results and provenance

**Do**

1. From the analyses list, click a **Succeeded** row (or **View results** on the
   dataset page).

**Expected, on the analysis page:**

- **Job status** card: status, and **Attempts** shown as `X / Y`, plus
  requested/started/completed timestamps.
- **Dataset** card: provenance metadata including the SHA-256 checksum.
- **Signal visualisation** for the source recording.
- **Provenance** card: input checksum, sampling frequency, channel and sample
  counts.
- **Per-channel results**: **Mean**, **Std. deviation**, **Minimum**,
  **Maximum**, **Peak-to-peak**, **Valid samples** per channel.

**Expected:** for the four-channel fixture each channel shows independent
statistics; failed or cancelled jobs show a safe "No results available" card and
**no** fabricated values.

**Talking point:** results carry dataset checksum, algorithm version,
parameters, and result-schema version, so a run is reproducible and
attributable.

---

### Scene 12 — Settings: organisation and profile

**Do**

1. Click **Organisation** in the navigation.
   - Show **Name**, **Status**, **Your role**, **Organisation ID**, and the
     **Members** list with each member's role.
2. Open the account menu (top-right) and click **Profile**.
   - Show **Display name**, **Email**, **Account ID**, and **Memberships** with
     roles.
3. Open the user menu again, click **Sign out**, then sign back in to confirm the
   round trip.

**Expected:** members and roles render; the UI hides write controls for viewers.
Sign-out returns to `/login`.

**Talking point:** role enforcement is server-side; the UI hiding an action is a
convenience, not the security boundary.

---

### Scene 13 — Tenant isolation (security)

**Narration:** "Identifiers in a URL never grant access."

**Do** (API)

```bash
# Two different development identities
TOKEN_A=$(curl -s -X POST http://localhost:8000/api/v1/auth/token \
  -H 'Content-Type: application/json' \
  -d '{"subject":"demo-a@eeg-lab.local"}' | python3 -c 'import sys,json;print(json.load(sys.stdin)["access_token"])')
TOKEN_B=$(curl -s -X POST http://localhost:8000/api/v1/auth/token \
  -H 'Content-Type: application/json' \
  -d '{"subject":"demo-b@eeg-lab.local"}' | python3 -c 'import sys,json;print(json.load(sys.stdin)["access_token"])')

# Organisation A's id
ORG_A=$(curl -s http://localhost:8000/api/v1/organisations \
  -H "Authorization: Bearer $TOKEN_A" | python3 -c 'import sys,json;print(json.load(sys.stdin)["items"][0]["id"])')

# Identity B tries to read A's organisation-scoped resources
curl -s -o /dev/null -w '%{http_code}\n' \
  http://localhost:8000/api/v1/organisations/$ORG_A/studies \
  -H "Authorization: Bearer $TOKEN_B"
```

**Expected:** `404` (not `403`) — the API prefers not to disclose that the
resource exists. Unauthenticated requests return `401`.

---

## 5. Optional — API-only quick tour

Mint a token, then walk the same workflow over REST. The BFF and the UI call
exactly these endpoints.

```bash
API=http://localhost:8000/api/v1
TOKEN=$(curl -s -X POST $API/auth/token -H 'Content-Type: application/json' \
  -d '{"subject":"demo-api@eeg-lab.local","display_name":"API Demo"}' \
  | python3 -c 'import sys,json;print(json.load(sys.stdin)["access_token"])')
AUTH="Authorization: Bearer $TOKEN"

ORG=$(curl -s $API/organisations -H "$AUTH" | python3 -c 'import sys,json;print(json.load(sys.stdin)["items"][0]["id"])')

# Organisation dashboard summary (counts only)
curl -s $API/organisations/$ORG/summary -H "$AUTH"

# Create a study
STUDY=$(curl -s -X POST $API/organisations/$ORG/studies -H "$AUTH" \
  -H 'Content-Type: application/json' \
  -d '{"name":"API Demo Study","description":"Created via REST"}' \
  | python3 -c 'import sys,json;print(json.load(sys.stdin)["id"])')

# Register a dataset (starts in pending_upload)
DATASET=$(curl -s -X POST $API/organisations/$ORG/studies/$STUDY/datasets -H "$AUTH" \
  -H 'Content-Type: application/json' \
  -d '{"name":"API Recording","format":"npy","source_description":"synthetic"}' \
  | python3 -c 'import sys,json;print(json.load(sys.stdin)["id"])')

# Create an upload session
SESSION=$(curl -s -X POST $API/organisations/$ORG/datasets/$DATASET/upload-sessions -H "$AUTH" \
  -H 'Content-Type: application/json' \
  -d '{"file_name":"resting-state-eeg.npy","file_size_bytes":65664,"format":"npy",
       "manifest":{"sampling_frequency_hz":256.0,
                   "channel_names":["EEG 001","EEG 002","EEG 003","EEG 004"],
                   "channel_units":["uV","uV","uV","uV"]}}' \
  | python3 -c 'import sys,json;print(json.load(sys.stdin)["upload_session_id"])')

# Upload the recording bytes, then finalise (validates -> ready)
curl -s -o /dev/null -X PUT \
  "$API/organisations/$ORG/datasets/$DATASET/upload-sessions/$SESSION/content" \
  -H "$AUTH" -H 'Content-Type: application/octet-stream' \
  --data-binary @demo/fixtures/resting-state-eeg.npy

curl -s -X POST \
  "$API/organisations/$ORG/datasets/$DATASET/upload-sessions/$SESSION/finalise" \
  -H "$AUTH"        # response dataset.status == "ready"

# Submit an analysis (idempotent on the key)
KEY=api-demo-$(date +%s)
JOB=$(curl -s -X POST $API/organisations/$ORG/datasets/$DATASET/analyses -H "$AUTH" \
  -H 'Content-Type: application/json' -H "Idempotency-Key: $KEY" \
  -d '{"analysis_type":"descriptive_statistics","algorithm_version":"1.0.0",
       "parameters":{"include_mean":true,"include_standard_deviation":true,
                     "include_min_max":true,"include_peak_to_peak":true}}' \
  | python3 -c 'import sys,json;print(json.load(sys.stdin)["id"])')

# Same key -> same job id (deduplication)
curl -s -X POST $API/organisations/$ORG/datasets/$DATASET/analyses -H "$AUTH" \
  -H 'Content-Type: application/json' -H "Idempotency-Key: $KEY" \
  -d '{"analysis_type":"descriptive_statistics","algorithm_version":"1.0.0"}' \
  | python3 -c 'import sys,json;print("same job:", json.load(sys.stdin)["id"])'

# Poll, then read the persisted result and provenance
curl -s $API/organisations/$ORG/analyses/$JOB -H "$AUTH"
curl -s $API/organisations/$ORG/analyses/$JOB/results -H "$AUTH"

# Signal window (bounded interval, selected channels)
curl -s "$API/organisations/$ORG/datasets/$DATASET/signals?start_seconds=0&end_seconds=5&channels=0" -H "$AUTH"

# Organisation-wide analysis history
curl -s "$API/organisations/$ORG/analyses?status=succeeded" -H "$AUTH"
```

Endpoint reference: `docs/07-api-spec.md`.

---

## 6. Feature coverage checklist

Tick each item as it is shown on screen or in the terminal.

- [ ] Liveness and readiness (`/health/live`, `/health/ready`)
- [ ] Prometheus metrics (`/metrics`)
- [ ] Passwordless development sign-in and HttpOnly session cookie
- [ ] Organisation-scoped dashboard with counts and recent activity
- [ ] Organisation switcher and active-organisation context
- [ ] Study create with inline validation
- [ ] Study search, status filter, cursor pagination, empty and no-results states
- [ ] Study detail, edit, and archive (read-only semantics)
- [ ] Dataset registration (`pending_upload`)
- [ ] Upload session → content → validation stepper
- [ ] Checksum, channel, sample, duration and unit metadata after validation
- [ ] Dataset lifecycle (`pending_upload` → `ready`; rejection path)
- [ ] Interactive signal chart: channel select, time window, prev/next
- [ ] Downsampling indicator and accessible channel summary
- [ ] Analysis submission with `Idempotency-Key`
- [ ] Asynchronous job states (`queued`, `running`, `succeeded`, `failed`, `cancelled`)
- [ ] Idempotent re-submission returns the same job
- [ ] Cooperative cancellation of queued jobs
- [ ] Org-wide analyses list with status filter and pagination
- [ ] Per-channel descriptive statistics
- [ ] Provenance: dataset checksum, algorithm version, parameters, schema version
- [ ] Organisation members and roles; profile and memberships
- [ ] Tenant isolation returns `404` for cross-organisation access
- [ ] Accessibility: labelled forms, status text beyond colour, chart alternative

---

## 7. Teardown

```bash
# Stop the stack but keep data volumes
docker compose down

# Also remove the database and stored recordings
docker compose down -v
```

If you used Option A, run `docker compose -p eeg-lab down` instead.

---

## 8. Troubleshooting

| Symptom                        | Likely cause                        | Fix                                                    |
| ------------------------------ | ----------------------------------- | ------------------------------------------------------ |
| `/health/ready` not ready      | Postgres/Redis still starting       | Wait and retry; check `docker compose ps`              |
| Sign-in shows "not available"  | API not in `AUTH_MODE=development`  | Use the Compose stack (dev mode)                       |
| Upload stuck on **Validation** | Worker/dispatcher not running       | `docker compose ps`; restart `worker` and `dispatcher` |
| Analysis stays **queued**      | Outbox not dispatched               | Ensure the `dispatcher` service is up                  |
| Empty dashboard                | User's organisation has no data yet | Continue the script to create a study                  |

---

## 9. Related material

- `demo/eeg-lab-demo.mp4` — recorded walkthrough.
- `demo/fixtures/` — deterministic recording and manifest.
- `scripts/create_demo.sh` — automation that produced the MP4.
- `packages/frontend/e2e/analysis-workflow.spec.ts` — API-level end-to-end tests.
- `docs/03-user-stories.md`, `docs/04-features.md` — normative behaviour.
