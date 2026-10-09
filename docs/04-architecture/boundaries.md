## 5. Repository and Module Boundaries

A suggested backend structure:

```text
packages/backend/app/
├── api/
│   ├── dependencies.py
│   ├── errors.py
│   ├── routes/
│   │   ├── auth.py
│   │   ├── organisations.py
│   │   ├── studies.py
│   │   ├── datasets.py
│   │   ├── signals.py
│   │   └── analyses.py
├── core/
│   ├── config.py
│   ├── logging.py
│   ├── security.py
│   └── exceptions.py
├── models/
│   ├── organisation.py
│   ├── study.py
│   ├── dataset.py
│   └── analysis.py
├── schemas/
│   ├── common.py
│   ├── study.py
│   ├── dataset.py
│   └── analysis.py
├── repositories/
│   ├── organisations.py
│   ├── studies.py
│   ├── datasets.py
│   └── analyses.py
├── services/
│   ├── studies.py
│   ├── uploads.py
│   ├── validation.py
│   ├── signals.py
│   └── analyses.py
├── scientific/
│   ├── recording.py
│   └── statistics.py
├── storage/
│   └── objects.py
├── workers/
│   ├── celery_app.py
│   └── analysis_tasks.py
└── main.py
```

This is a proposed organisation. Keep the codebase simpler if a module does not yet justify its own file.

Avoid circular imports and avoid importing API route handlers into business logic.

