## 14. Dependency Management

### Python

Use `uv` for Python project and dependency management.

Maintain a committed lockfile and a documented Python version.

Separate runtime dependencies from development and testing dependencies.

### TypeScript

Use `pnpm` for JavaScript dependency management.

Commit the lockfile and use frozen-lockfile installation in CI.

### Git hooks

Use Husky to install the repository's Git hooks through the root `prepare` script. The `pre-commit` hook runs `lint-staged`, which applies Ruff lint fixes and formatting to staged Python files and ESLint plus Prettier to staged frontend JavaScript/TypeScript files. Install dependencies with `pnpm install` to configure hooks; CI remains responsible for full tests, type checks, and builds.

### Version policy

- Pin runtime versions deliberately.
- Avoid floating production image tags.
- Review security advisories.
- Upgrade dependencies through tested pull requests.
- Avoid adding multiple libraries for the same responsibility.
- Document important version constraints.

