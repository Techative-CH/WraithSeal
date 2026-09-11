# WraithSeal Vault Format

## Table of Contents

1. [Introduction](#1-introduction)
2. [Format Design Goals](#2-format-design-goals)

---

## 1. Introduction

This document defines the persistent format used to represent a WraithSeal
vault and its security-critical metadata.

The vault format translates the cryptographic design into a deterministic
serialized representation that can be stored, validated, updated, and
interpreted consistently across implementations.

The format must preserve the cryptographic relationships defined by the system
architecture and cryptographic design, including:

- Vault identity.
- Vault format version.
- Cryptographic design version.
- Security generation.
- Password derivation configuration.
- Possession-factor metadata.
- Normal and recovery VMK wrappers.
- Authenticated cryptographic context.
- Encrypted vault data.

The format does not define operating-system-specific mounting behaviour or
user-interface behaviour.

---

## 2. Format Design Goals

The vault format must satisfy the following properties:

- The format must be unambiguously identifiable as a WraithSeal vault.
- The serialized representation must be deterministic where cryptographic processing depends on byte-level encoding.
- Security-critical fields must have explicit types and lengths.
- Cryptographic parameters must be stored explicitly.
- The format must support forward versioning without silently changing the interpretation of existing vaults.
- Corrupted, incomplete, or unsupported vault data must be detectable.
- Security metadata belonging to one vault must not be valid when transplanted into another vault.
- Security metadata belonging to one security generation must not be accepted as current metadata for another generation.
- Normal and recovery VMK wrappers must remain independently identifiable.
- Security-state updates must not expose partially committed authoritative state.
- The VMK, PDK, NUK, and RUK must never be stored in plaintext.
- The format must permit credential rotation without requiring the complete encrypted data region to be rewritten.
