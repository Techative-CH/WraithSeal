# WraithSeal

> **Sealed until you’re there.**

WraithSeal is an open-source project for hardware-assisted encrypted storage.

The goal is to protect encrypted data using two separate factors:

- something the user **knows**, such as a password.
- something the user **possesses**, such as a removable USB device.

The removable device acts as a physical possession factor and is required as
part of the process used to unlock the encrypted data.

## Status

> [!WARNING]
> WraithSeal is currently in early development and is not ready for production use.

The security architecture, threat model, and cryptographic design are still
being defined.

## Goals

- Secure encrypted storage.
- Removable USB device as a physical possession factor.
- Password-based protection.
- Secure key management.
- Recovery mechanism.
- Simple and auditable design.
- Cross-platform support where practical.

## Security

WraithSeal will rely on established cryptographic primitives and libraries
rather than custom cryptography.

Security-related design decisions and the project threat model will be
documented as development progresses.

For information about reporting vulnerabilities, see
[SECURITY.md](SECURITY.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## License

WraithSeal is licensed under the [Apache License 2.0](LICENSE).

---

**WraithSeal** is developed by **Techative**.

Copyright © 2026 Samuel Banfi.