## 6. Information Architecture

Recommended route structure:

```text
/
├── login
├── app
│   ├── dashboard
│   ├── studies
│   │   ├── new
│   │   └── [studyId]
│   │       ├── settings
│   │       └── datasets
│   │           ├── upload
│   │           └── [datasetId]
│   │               ├── overview
│   │               ├── signals
│   │               └── analyses
│   ├── analyses
│   │   └── [jobId]
│   │       └── result
│   └── settings
│       ├── profile
│       └── organisation
└── not-found
```

Adapt this structure to the chosen frontend router. It is a logical route map, not a requirement to create empty pages for every path.

### Navigation rules

- Protected pages require authentication.
- Resource pages require server-verified access.
- Breadcrumbs should use human-readable resource names where available.
- Refreshing a deep link must preserve the intended destination if the user remains authorised.
- Unknown routes display a proper not-found page.
- Unauthorised and nonexistent resources should be handled consistently to avoid leaking resource existence.

