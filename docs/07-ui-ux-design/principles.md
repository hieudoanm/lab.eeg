## 2. Design Principles

### 2.1 Clarity before decoration

Every screen should make three things obvious:

1. Where am I?
2. What data am I looking at?
3. What can I do next?

Avoid decorative components that compete with scientific data.

### 2.2 Data density with readable hierarchy

Researchers need to inspect metadata, tables, job states, and charts efficiently.

Use compact layouts for scientific information while preserving readable text, clear labels, and sufficient spacing.

### 2.3 Scientific precision

Display measurement units, timestamps, channel names, processing states, and provenance wherever relevant.

Never imply that an approximate display representation is the original raw recording.

### 2.4 Honest system feedback

Every asynchronous operation must communicate its state. Do not show a success notification before the server confirms success.

### 2.5 Consistent interactions

Use the same patterns for navigation, forms, status badges, dialogs, tables, errors, and confirmations throughout the application.

### 2.6 Security-aware presentation

Do not expose resources, identifiers, metadata, or actions that the current user is not authorised to access.

Frontend visibility is a usability measure, not a substitute for server-side authorisation.

