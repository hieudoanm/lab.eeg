## 15. Forms, Dialogs, and Feedback

### Forms

Every form should have:

- A clear title.
- Labels for every field.
- Required-field indicators where appropriate.
- Inline validation.
- Disabled or progress state during submission.
- A clear success or failure outcome.

Preserve entered values after recoverable failures.

### Dialogs

Use dialogs for short, focused actions such as confirming an archive operation.

Do not place complex multi-step scientific workflows inside small dialogs when a full page would be clearer.

### Notifications

Use non-blocking notifications for routine success or failure messages.

For important validation failures, show the explanation in the page or form as well.

Notifications should:

- Be announced appropriately to assistive technologies.
- Remain readable.
- Avoid conveying essential information through colour alone.
- Never claim success before the server confirms it.

### Destructive actions

Clearly identify the affected resource and the consequence of the operation.

Use a confirmation step when the operation is destructive or difficult to reverse. Prefer archive or reversible operations over permanent deletion where appropriate.

