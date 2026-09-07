# Contributing to WraithSeal

Thank you for your interest in contributing to WraithSeal.

WraithSeal is a security-focused project. Contributions should prioritize security, correctness, maintainability, and clarity.

## Development Workflow

The repository follows a branch-based development workflow.

### Branches

- `main` contains stable and release-ready code.
- `development` is the primary integration branch for ongoing development.
- Feature and fix branches are created from `development`.

Do not commit directly to `main` or `development`.

### Creating a Branch

Create a new branch from the latest `development` branch.

Use descriptive branch names with one of the following prefixes:

- `feature/` - new functionality.
- `fix/` - bug fixes.
- `refactor/` - internal code improvements.
- `docs/` - documentation changes.
- `test/` - test-related changes.
- `chore/` - maintenance and tooling.

Examples:

    feature/usb-enrollment
    feature/vault-initialization
    fix/device-detection
    docs/threat-model

## Merge Requests

All changes should be integrated through GitLab Merge Requests.

The normal development flow is:

    feature/* → development → main

Feature, fix, and other development branches should target `development`.

Changes from `development` are merged into `main` when they are considered stable and ready for release.

Before submitting a Merge Request:

- Ensure the project builds successfully.
- Run all relevant tests.
- Keep the change focused on a single purpose.
- Update documentation when necessary.
- Do not include secrets, credentials, private keys, or sensitive data.

## Commits

Write concise and descriptive commit messages.

Examples:

    Add USB device enrollment
    Implement vault initialization
    Fix removable device detection
    Update threat model

Avoid vague commit messages such as:

    Update
    Changes
    Fix stuff
    WIP

## Security

Security is a core requirement of WraithSeal.

Contributions involving cryptography, key management, authentication, sensitive memory, storage, or device identification require particular care.

Do not introduce custom cryptographic algorithms or primitives.

Use established and well-reviewed cryptographic libraries and document security-sensitive design decisions.

Never commit:

- passwords.
- API tokens.
- private keys.
- recovery keys.
- encryption keys.
- real user secrets.
- sensitive test data.

If you discover a security vulnerability, do not disclose it publicly through an issue or Merge Request. Follow the process described in `SECURITY.md`.

## Documentation

Security-sensitive behavior and architectural decisions should be documented.

Relevant changes may require updates to:

- the README.
- architecture documentation.
- the threat model.
- security documentation.
- user documentation.

## Testing

New functionality should include appropriate tests whenever possible.

Security-critical functionality should not be merged without tests covering its expected behavior and relevant failure cases.

## License

By contributing to WraithSeal, you agree that your contributions will be licensed under the Apache License 2.0.