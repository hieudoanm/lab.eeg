## 3. Environment Strategy

Use three logical environments.

| Environment     | Purpose                        | Data                        |
| --------------- | ------------------------------ | --------------------------- |
| Local           | Daily development              | Synthetic data              |
| CI              | Automated testing              | Isolated synthetic fixtures |
| Production demo | Public portfolio demonstration | Synthetic data only         |

A separate staging environment may be introduced when the deployment complexity justifies it.

### Environment isolation

Each environment should have its own:

- Configuration.
- Database.
- Object-storage location.
- Authentication configuration.
- Service identities.
- Secrets.
- Logging context.

Do not let CI or local development accidentally write to the production database or production storage bucket.

