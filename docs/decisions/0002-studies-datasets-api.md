# 0002 — Studies and datasets API shape

- Status: Accepted
- Date: 2026-10-10
- Phase: 4 (Studies and Datasets)

## Context

`docs/07-api-spec.md` §8–§9 define the study and dataset REST contract but leave two
points open:

- The study list documents `search` as "optional name search, if implemented",
  without specifying matching semantics.
- The dataset registration example includes a free-form `metadata` object and a
  `source_description`, but the response shapes do not list every stored column.

`docs/04-features.md` §4 also shows a "number of datasets" column in the study
list. The API contract in §8 does not include a dataset count in the study
representation.

Separately, the frontend `Study` and `Dataset` types carried an
`organisation_id` field that the API has never returned and the spec never
documents.

## Decision

- **Search** is implemented as a case-insensitive substring match on the study
  name (`name ILIKE %term%`). LIKE wildcards (`%`, `_`, `\`) in user input are
  escaped with an explicit `ESCAPE '\'` clause so a search term is matched
  literally. The filter is always combined with the verified organisation scope
  and is applied before the keyset cursor, preserving deterministic ordering.
- **Study and dataset responses never include `organisation_id`.** The tenant is
  already established by the scoped path and verified membership; echoing it adds
  no information to the caller and would leak organisation identifiers across a
  shared UI. The field is removed from the frontend types to match the spec and
  the backend schemas.
- **The study representation does not include a dataset count.** The spec is
  normative for the response shape; a count would require a join or subquery and
  a documented contract change. The study detail page lists datasets directly,
  so the count is deferred until it is added to the spec.
- **Dataset `storage_key` and `metadata` are never returned.** The registration
  endpoint accepts `metadata` and `source_description`, but only approved
  columns are mapped onto `DatasetRead`; `storage_key` stays server-side.

## Consequences

- A client can search studies predictably; a literal `%` or `_` in a term does
  not broaden the match.
- The frontend and backend contracts stay aligned with `docs/07-api-spec.md`; no
  organisation identifier is exposed in study or dataset payloads.
- Phase 4 ships without the optional dataset-count column. Adding it later
  requires a spec update plus a test, per the delivery rules.
- The dataset `metadata` object is persisted for provenance but is intentionally
  not part of the read contract yet; exposing it will need an explicit decision.
