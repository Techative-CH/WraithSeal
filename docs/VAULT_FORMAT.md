# WraithSeal Vault Format

## Table of Contents

1. [Introduction](#1-introduction)
2. [Format Design Goals](#2-format-design-goals)
3. [Vault File Structure](#3-vault-file-structure)
4. [File Header](#4-file-header)
5. [Vault Identity and Versioning](#5-vault-identity-and-versioning)
6. [Password Derivation Configuration](#6-password-derivation-configuration)
7. [Security Metadata](#7-security-metadata)
8. [VMK Wrappers](#8-vmk-wrappers)
9. [Possession-Factor Metadata](#9-possession-factor-metadata)
10. [Encrypted Data Region](#10-encrypted-data-region)
11. [Canonical Encoding](#11-canonical-encoding)
12. [Integrity and Authentication](#12-integrity-and-authentication)
13. [Atomic Updates](#13-atomic-updates)
14. [Recovery Material Format](#14-recovery-material-format)
15. [Validation and Failure Behaviour](#15-validation-and-failure-behaviour)
16. [Glossary](#16-glossary)

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

---

## 3. Vault File Structure

A vault is represented as a single binary file using the `.ws` extension.

Example:

```text
myvault.ws
```

The file is logically divided into the following top-level regions:

- Fixed Header.
- Security Metadata Slot A.
- Security Metadata Slot B.
- Encrypted Data Region.

The Fixed Header identifies the file as a valid vault and provides the
structural information required to locate and interpret the remaining regions.

Each Security Metadata slot can contain one complete persistent security
configuration.

At any given time, one complete valid slot represents the current authoritative
Security Generation. The other slot may contain a previous generation, a
candidate generation being prepared, or no valid Security Metadata.

Each complete Security Metadata structure is logically composed of:

- Metadata Header.
- Password Derivation Configuration.
- Possession-Factor Metadata.
- Normal VMK Wrapper.
- Recovery VMK Wrapper.

The Normal VMK Wrapper and Recovery VMK Wrapper stored within the same metadata
slot contain independently protected representations of the same Vault Master
Key and belong to the same Security Generation.

The Encrypted Data Region contains the encrypted representation of the vault
contents.

The exact byte layout, field lengths, offsets, and canonical encoding rules are
defined by the Vault Format Version.

---

## 4. File Header

Every vault begins with a fixed-size 128-byte header.

The header identifies the file as a WraithSeal vault and provides the
structural information required to locate and interpret the persistent regions
of the container.

The header layout for vault format version 1 is:

| Offset |     Size | Field                  | Description                                           |
| -----: | -------: | ---------------------- | ----------------------------------------------------- |
| `0x00` |  8 bytes | Magic                  | Identifies the file as a WraithSeal vault.            |
| `0x08` |  2 bytes | Format Version         | Identifies the vault format version.                  |
| `0x0A` |  2 bytes | Header Size            | Size of the fixed header in bytes.                    |
| `0x0C` |  4 bytes | Flags                  | Format-level feature flags.                           |
| `0x10` | 16 bytes | Vault ID               | Random identifier uniquely associated with the vault. |
| `0x20` |  8 bytes | Metadata Slot A Offset | Absolute byte offset of Security Metadata Slot A.     |
| `0x28` |  8 bytes | Metadata Slot A Length | Length of Security Metadata Slot A in bytes.          |
| `0x30` |  8 bytes | Metadata Slot B Offset | Absolute byte offset of Security Metadata Slot B.     |
| `0x38` |  8 bytes | Metadata Slot B Length | Length of Security Metadata Slot B in bytes.          |
| `0x40` |  8 bytes | Encrypted Data Offset  | Absolute byte offset of the Encrypted Data Region.    |
| `0x48` |  8 bytes | Encrypted Data Length  | Length of the Encrypted Data Region in bytes.         |
| `0x50` | 48 bytes | Reserved               | Reserved for future format extensions.                |

The total header size is 128 bytes.

### 4.1 Magic

The first eight bytes contain the following magic value:

```text
57 52 41 49 54 48 53 00
```

which corresponds to:

```text
WRAITHS\0
```

The magic value provides file-type identification only and does not provide
cryptographic authentication.

A file with an invalid magic value must not be interpreted as a WraithSeal
vault.

### 4.2 Format Version

The Format Version identifies the byte-level interpretation of the vault
container.

Vault format version 1 is represented by the integer value `1`.

An implementation must reject a vault whose format version it does not
support rather than attempting to interpret it using another version.

### 4.3 Header Size

The Header Size field contains the size of the fixed header in bytes.

For format version 1, this value is:

```text
128
```

The field is stored explicitly so that future format versions can define a
different header structure without requiring version 1 parsers to infer its
size.

### 4.4 Flags

The Flags field is a 32-bit bit field reserved for format-level features.

No flags are currently defined for format version 1.

All bits must therefore be zero when a version 1 vault is created.

A version 1 implementation must reject unsupported non-zero flag bits rather
than silently ignoring them.

### 4.5 Vault ID

The Vault ID is a randomly generated 128-bit identifier assigned when the
vault is created.

It remains stable for the lifetime of the vault and is used as part of the
cryptographic binding between persistent security material and the vault to
which that material belongs.

The Vault ID is not secret.

### 4.6 Region Location Fields

The header contains the absolute byte offset and byte length of the persistent
regions required to interpret the vault:

- Metadata Slot A Offset.
- Metadata Slot A Length.
- Metadata Slot B Offset.
- Metadata Slot B Length.
- Encrypted Data Offset.
- Encrypted Data Length.

Offsets are measured from the beginning of the `.ws` file.

Metadata Slot A and Metadata Slot B provide independent storage locations for
complete Security Metadata structures. They support security-state transitions
without overwriting the currently authoritative Security Metadata in place.

The Fixed Header does not identify either metadata slot as authoritative.
Authoritative state is determined from the validity and Security Generation of
the complete Security Metadata structures stored in the slots.

A parser must validate all offsets and lengths before using them.

The metadata slots and Encrypted Data Region must not overlap each other or the
Fixed Header. No region may extend beyond the physical end of the file or
produce an integer overflow when its boundaries are calculated.

### 4.7 Reserved Bytes

Bytes from offset `0x50` through `0x7F` are reserved for future format
extensions.

For vault format version 1, all Reserved bytes must be zero.

Version 1 implementations must not assign semantic meaning to these bytes.

---

## 5. Vault Identity and Versioning

The vault format distinguishes between persistent vault identity, container
format versioning, cryptographic design versioning, and security-state
generation.

These values serve different purposes and must not be treated as
interchangeable.

### 5.1 Vault ID

Each vault has a randomly generated 128-bit Vault ID.

The Vault ID is created once when the vault is created and remains stable for
the lifetime of that vault.

It is used to bind cryptographic material and security metadata to the vault
for which they were created.

The Vault ID:

- Is not secret.
- Must not change during password changes, possession-factor replacement,
  recovery-material rotation, or recovery.
- Must be included in cryptographic contexts where vault-specific binding is
  required.
- Must not be reused when a new independent vault is created.

Copying or renaming a `.ws` file does not create a new vault identity. The
copied file retains the same Vault ID.

### 5.2 Vault Format Version

The Vault Format Version identifies the byte-level structure and serialization
rules of the `.ws` container.

Format version `1` defines the layout documented by this specification.

The format version determines how an implementation interprets:

- The fixed header.
- Region structure.
- Field sizes.
- Integer encoding.
- Security metadata serialization.
- Wrapper serialization.
- Encrypted data layout.

An implementation must reject unsupported format versions.

A Vault Format Version change is required when the persistent representation
changes in a way that cannot be interpreted safely using the existing format
definition.

The Vault Format Version does not identify the current credentials or security
generation of a vault.

### 5.3 Cryptographic Design Version

The Cryptographic Design Version identifies the cryptographic construction used
to interpret security-critical derivations and protected key material.

It is independent from the Vault Format Version.

For example, the Cryptographic Design Version determines the expected meaning
of:

- Password-based key derivation.
- Normal Unlock Key derivation.
- Recovery Unlock Key derivation.
- Domain-separation contexts.
- VMK wrapper protection.
- Associated-data construction.

A future implementation may therefore support multiple cryptographic designs
within the same general container format.

The Cryptographic Design Version is stored in authenticated security metadata
and participates in the cryptographic context associated with VMK protection.

An implementation must not reinterpret security material using a cryptographic
design version different from the one recorded for that security state.

### 5.4 Security Generation

The Security Generation identifies the current security configuration of a
vault.

It is an unsigned monotonic integer associated with security-sensitive
credential state.

The initial security configuration is generation `1`.

A successful security-state transition increments the generation:

```text
Generation N -> Generation N+1
```

Transitions that advance the Security Generation include:

- Password change.
- Possession-factor replacement.
- Recovery-material rotation.
- Recovery.

The Security Generation does not change when vault contents are modified
without changing the security configuration.

The current generation participates in:

- Normal unlock key derivation.
- Recovery unlock key derivation.
- Normal VMK wrapper authentication.
- Recovery VMK wrapper authentication.
- Security-state binding.

Material belonging to generation `N` must not authenticate against the current
authoritative state after generation `N+1` has been committed.

The Security Generation is stored in authenticated Security Metadata rather
than in the fixed file header.

### 5.5 Version Independence

The following values are independent:

| Value                        | Changes When                                                  |
| ---------------------------- | ------------------------------------------------------------- |
| Vault ID                     | A new independent vault is created.                           |
| Vault Format Version         | The persistent container representation changes incompatibly. |
| Cryptographic Design Version | The cryptographic construction or interpretation changes.     |
| Security Generation          | The vault security configuration changes.                     |

For example, changing a password increments the Security Generation but does
not create a new Vault ID, Vault Format Version, or Cryptographic Design
Version.

Similarly, modifying files stored inside the vault does not by itself change
any of these values except persistent data associated with the encrypted
contents.

---

## 6. Password Derivation Configuration

The Password Derivation Configuration contains the persistent parameters
required to reproduce the Password-Derived Key (PDK).

The configuration is stored as part of the authenticated Security Metadata.

For vault format version 1, the configuration uses the following layout:

|     Size | Field          | Description                                             |
| -------: | -------------- | ------------------------------------------------------- |
|  2 bytes | KDF Algorithm  | Identifies the password-based key derivation algorithm. |
|  2 bytes | Argon2 Version | Identifies the Argon2 specification version.            |
|  4 bytes | Memory Cost    | Argon2 memory cost.                                     |
|  4 bytes | Time Cost      | Argon2 iteration count.                                 |
|  4 bytes | Parallelism    | Argon2 parallelism parameter.                           |
|  2 bytes | Output Length  | Length of the derived PDK in bytes.                     |
|  2 bytes | Salt Length    | Length of the password salt in bytes.                   |
| 16 bytes | Password Salt  | Random salt used for password derivation.               |

The total serialized size is 36 bytes.

### 6.1 KDF Algorithm

The KDF Algorithm field identifies the password-based key derivation algorithm
used by the vault.

For format version 1:

```text
0x0001 = Argon2id
```

An implementation must reject an unsupported KDF identifier rather than
interpreting the remaining parameters using another algorithm.

### 6.2 Argon2 Version

The Argon2 Version field identifies the Argon2 specification version required
to reproduce the PDK.

For the current cryptographic design, Argon2 version `0x13` is used,
corresponding to Argon2 version 1.3.

The version must be stored explicitly and must not be inferred from a
cryptographic-library default.

### 6.3 Memory Cost

The Memory Cost field stores the Argon2 memory parameter.

The value is expressed in kibibytes.

The minimum permitted value for vault format version 1 is:

```text
65536
```

corresponding to 64 MiB.

A vault may specify a higher value.

An implementation must not silently reduce this parameter when opening an
existing vault.

### 6.4 Time Cost

The Time Cost field stores the number of Argon2 iterations.

The minimum permitted value for vault format version 1 is:

```text
3
```

A vault may specify a higher value.

### 6.5 Parallelism

The Parallelism field stores the Argon2 parallelism parameter.

The minimum configuration defined by the cryptographic design uses:

```text
4
```

Implementations must reproduce the stored value when deriving the PDK.

### 6.6 Output Length

The Output Length field stores the required PDK size in bytes.

For vault format version 1, this value must be:

```text
32
```

corresponding to a 256-bit PDK.

A version 1 implementation must reject any other value.

### 6.7 Password Salt

Each vault security configuration contains a randomly generated password salt.

For format version 1:

```text
Salt Length = 16 bytes
```

The Password Salt is generated using the system CSPRNG whenever a new password
derivation configuration is created.

The salt is not secret.

The Salt Length field must contain `16` for vault format version 1.

### 6.8 Validation

Before performing password derivation, an implementation must validate the
complete Password Derivation Configuration.

At minimum, validation must reject:

- Unsupported KDF algorithms.
- Unsupported Argon2 versions.
- Memory cost below the minimum permitted value.
- Time cost below the minimum permitted value.
- Invalid parallelism values.
- Output lengths other than 32 bytes.
- Salt lengths other than 16 bytes.
- Truncated or incomplete parameter blocks.

Invalid password-derivation parameters must be treated as invalid security
metadata and must not be interpreted as an authentication failure.

---

## 7. Security Metadata

Each Security Metadata slot contains one complete Security Metadata structure
required to authenticate access to the vault and recover the Vault Master Key.

A complete metadata structure represents exactly one Security Generation. It
may represent the current authoritative state, a previous generation retained
in the alternate slot, or a candidate generation being prepared during a
security-state transition.

The structure contains:

```text
Security Metadata
├── Metadata Header
├── Password Derivation Configuration
├── Possession-Factor Metadata
├── Normal VMK Wrapper
└── Recovery VMK Wrapper
```

The Security Metadata must be treated as a complete security state. Individual
security-critical components must not be independently replaced in the
authoritative state.

### 7.1 Metadata Header

The Metadata Header identifies and describes the security configuration stored
in the structure.

For vault format version 1, the Metadata Header has a fixed serialized size of
16 bytes.

The layout is:

| Offset |    Size | Field                        | Description                                                            |
| -----: | ------: | ---------------------------- | ---------------------------------------------------------------------- |
| `0x00` | 8 bytes | Security Generation          | Identifies the current security configuration.                         |
| `0x08` | 2 bytes | Cryptographic Design Version | Identifies the cryptographic construction used by the security state.  |
| `0x0A` | 2 bytes | Metadata Version             | Identifies the serialization rules of the Security Metadata structure. |
| `0x0C` | 4 bytes | Metadata Length              | Total serialized length of the Security Metadata structure.            |

For vault format version 1, the complete Security Metadata structure has a
fixed serialized size of `224` bytes:

```text
16 + 36 + 20 + 76 + 76 = 224 bytes
```

The Metadata Length field must therefore contain `224`.

All multi-byte integer fields use the canonical big-endian encoding defined in
Section 11.

### 7.2 Security Generation

The Security Generation stored in Security Metadata identifies the generation
to which all security-critical material in the structure belongs.

The initial Security Generation is:

```text
1
```

All components contained in the same Security Metadata structure must belong to
the same generation.

This includes:

- The password derivation configuration.
- The possession-factor configuration.
- The Normal VMK Wrapper.
- The Recovery VMK Wrapper.
- Cryptographic contexts associated with those wrappers.

A security-state transition prepares a complete replacement Security Metadata
structure for generation `N+1` before it becomes authoritative.

Security-critical material from different generations must not be combined to
construct an authoritative security state.

### 7.3 Cryptographic Design Version

The Cryptographic Design Version identifies the cryptographic construction
required to interpret the security configuration.

For the initial cryptographic design, the version is:

```text
1
```

The value determines the expected interpretation of:

- Password derivation.
- Normal Unlock Key derivation.
- Recovery Unlock Key derivation.
- Domain-separation contexts.
- VMK wrapper protection.
- Associated-data construction.

An implementation must reject an unsupported Cryptographic Design Version.

The value is included in authenticated cryptographic context and must not be
silently substituted with another supported version.

### 7.4 Metadata Version

The Metadata Version identifies the serialization structure of the Security
Metadata structure.

For vault format version 1, the initial Metadata Version is:

```text
1
```

The Metadata Version is independent from:

- The Vault Format Version.
- The Cryptographic Design Version.
- The Security Generation.

A Metadata Version change is required when the structure or interpretation of
the Security Metadata structure changes incompatibly without requiring a complete
change to the surrounding vault container format.

An implementation must reject unsupported Metadata Versions.

### 7.5 Password Derivation Configuration

The Security Metadata contains exactly one Password Derivation Configuration
for the current security generation.

The configuration is serialized according to Section 6.

It provides all persistent information required to derive the PDK from the
user-supplied password.

### 7.6 Possession-Factor Metadata

The Security Metadata contains the persistent non-secret information required
to identify and validate the possession factor associated with the current
normal authentication path.

The Possession Secret itself is not stored in the `.ws` file.

Possession-factor metadata is defined in Section 9.

### 7.7 VMK Wrappers

The Security Metadata contains exactly two independently protected
representations of the same VMK:

- One Normal VMK Wrapper.
- One Recovery VMK Wrapper.

The Normal VMK Wrapper belongs exclusively to the normal authentication path.

The Recovery VMK Wrapper belongs exclusively to the recovery authentication
path.

Neither wrapper contains the VMK in plaintext.

Both wrappers must belong to the same Vault ID, Security Generation,
Cryptographic Design Version, and authoritative security state.

Their serialized representation is defined in Section 8.

### 7.8 Recovery Secret Exclusion

The Recovery Secret is not stored in the `.ws` file.

It exists only as part of the separately stored recovery material.

The vault contains the Recovery VMK Wrapper and the non-secret information
required to interpret it, but recovery must remain impossible without both:

- The correct password.
- The corresponding external recovery material.

### 7.9 Security Metadata Validation

Security Metadata must be structurally validated before it is used for
cryptographic authentication.

Validation must reject at least:

- Unsupported Metadata Versions.
- Unsupported Cryptographic Design Versions.
- Invalid Security Generation values.
- Metadata Length values other than `224` for vault format version 1.
- Invalid or incomplete Password Derivation Configuration.
- Invalid possession-factor metadata.
- Missing VMK wrappers.
- Duplicate VMK wrappers of the same type.
- Invalid wrapper lengths.
- Malformed or truncated fields.
- Structure lengths inconsistent with the serialized metadata.
- Security-critical components that cannot be interpreted as belonging to the
  same security state.

Structural validation does not replace cryptographic authentication.

Metadata that is structurally valid must still satisfy the authenticated
cryptographic checks defined by the VMK wrapper construction before a VMK can
be accepted.

---

## 8. VMK Wrappers

The Vault Master Key is stored only in protected form.

Each complete Security Metadata structure contains exactly two VMK wrappers:

- One Normal VMK Wrapper.
- One Recovery VMK Wrapper.

Both wrappers protect the same 256-bit VMK but use independently derived keys,
independent nonces, and distinct authenticated contexts.

For vault format version 1, each VMK wrapper has a fixed serialized size of
76 bytes.

The wrapper layout is:

|     Size | Field              | Description                                                     |
| -------: | ------------------ | --------------------------------------------------------------- |
|   1 byte | Wrapper Type       | Identifies the authentication path associated with the wrapper. |
|  3 bytes | Reserved           | Reserved for future format extensions.                          |
| 24 bytes | Nonce              | XChaCha20-Poly1305 nonce.                                       |
| 32 bytes | VMK Ciphertext     | Encrypted representation of the 256-bit VMK.                    |
| 16 bytes | Authentication Tag | Poly1305 authentication tag.                                    |

### 8.1 Wrapper Type

The Wrapper Type identifies the purpose of the protected VMK representation.

For vault format version 1:

```text
0x01 = Normal VMK Wrapper
0x02 = Recovery VMK Wrapper
```

A complete Security Metadata structure must contain exactly one wrapper of each type.

An implementation must reject unknown wrapper types.

The Wrapper Type participates in the authenticated associated data and must
therefore not be interpreted independently from the cryptographic context of
the wrapper.

### 8.2 Reserved Bytes

The three Reserved bytes are reserved for future format extensions.

For vault format version 1, all Reserved bytes must be zero.

Version 1 implementations must reject a wrapper containing non-zero Reserved
bytes.

### 8.3 Nonce

Each wrapper contains a 192-bit nonce used by XChaCha20-Poly1305.

The nonce is generated independently using the system CSPRNG when the wrapper
is created.

A nonce must never be intentionally reused with the same encryption key.

The Normal VMK Wrapper and Recovery VMK Wrapper use independently generated
nonces.

The nonce is not secret and is stored directly with its corresponding wrapper.

### 8.4 VMK Ciphertext

The VMK Ciphertext field contains the encrypted 256-bit Vault Master Key.

For a version 1 vault, the plaintext VMK is exactly 32 bytes and therefore the
ciphertext portion is also exactly 32 bytes.

The VMK must never be serialized in plaintext.

The Normal VMK Wrapper encrypts the VMK using the NUK.

The Recovery VMK Wrapper encrypts the same VMK using the RUK.

### 8.5 Authentication Tag

Each wrapper contains the 128-bit authentication tag produced by
XChaCha20-Poly1305.

The tag authenticates both:

- The encrypted VMK.
- The associated data supplied to the wrapper operation.

A wrapper must not be accepted unless authenticated decryption successfully
verifies its authentication tag.

Authentication failure must not produce a usable VMK.

### 8.6 Associated Data

Each VMK wrapper is cryptographically bound to its non-secret security context
through authenticated associated data.

For vault format version 1, the associated data contains:

```text
Vault ID
Vault Format Version
Metadata Version
Cryptographic Design Version
Security Generation
Wrapper Type
Password Derivation Configuration
Possession-Factor Metadata
```

The Password Derivation Configuration and Possession-Factor Metadata are
included using their complete canonical serialized representations.

The values are encoded using the canonical encoding rules defined by this
format.

The Normal VMK Wrapper uses Wrapper Type `0x01`.

The Recovery VMK Wrapper uses Wrapper Type `0x02`.

The associated data is not stored as a duplicate serialized block inside each
wrapper. It is reconstructed deterministically from the authoritative vault
and Security Metadata fields when authenticated encryption or decryption is
performed.

Any modification to a value participating in the associated data must cause
wrapper authentication to fail.

### 8.7 Normal VMK Wrapper

The Normal VMK Wrapper is protected using the Normal Unlock Key:

```text
XChaCha20-Poly1305(
    key        = NUK,
    nonce      = Normal Nonce,
    plaintext  = VMK,
    aad        = Normal Wrapper AAD
)
-> Normal VMK Ciphertext + Authentication Tag
```

Successful authenticated decryption recovers the VMK for normal vault access.

### 8.8 Recovery VMK Wrapper

The Recovery VMK Wrapper is protected using the Recovery Unlock Key:

```text
XChaCha20-Poly1305(
    key        = RUK,
    nonce      = Recovery Nonce,
    plaintext  = VMK,
    aad        = Recovery Wrapper AAD
)
-> Recovery VMK Ciphertext + Authentication Tag
```

Successful authenticated decryption recovers the VMK only for the recovery
workflow.

Recovery must not expose vault contents merely because the Recovery Wrapped
VMK was successfully authenticated and decrypted.

### 8.9 Wrapper Validation

Before attempting authenticated decryption, an implementation must validate
the serialized wrapper structure.

Validation must reject at least:

- Unsupported Wrapper Types.
- Duplicate wrappers of the same type.
- Invalid Reserved bytes.
- Invalid nonce lengths.
- Invalid ciphertext lengths.
- Invalid authentication-tag lengths.
- Truncated wrappers.
- Wrappers inconsistent with the current Security Metadata.

Structural validation does not establish authenticity.

A structurally valid wrapper becomes cryptographically valid only after
successful XChaCha20-Poly1305 authenticated decryption using the expected key,
nonce, and associated data.

---

## 9. Possession-Factor Metadata

The Possession-Factor Metadata describes the possession factor associated with
the current normal authentication path.

The metadata stored in the `.ws` file does not contain the Possession Secret.

For vault format version 1, the Possession-Factor Metadata has a fixed
serialized size of 20 bytes.

The layout is:

|     Size | Field                | Description                                                       |
| -------: | -------------------- | ----------------------------------------------------------------- |
|   1 byte | Factor Type          | Identifies the type of possession factor.                         |
|  3 bytes | Reserved             | Reserved for future format extensions.                            |
| 16 bytes | Possession Factor ID | Random identifier associated with the enrolled possession factor. |

### 9.1 Factor Type

The Factor Type identifies the kind of possession factor expected by the
current security configuration.

For vault format version 1:

```text
0x01 = Removable storage device
```

An implementation must reject unsupported Factor Types.

The Factor Type describes how possession material is obtained. It does not
provide cryptographic authentication by itself.

### 9.2 Possession Factor ID

Each enrolled possession factor is assigned a randomly generated 128-bit
Possession Factor ID.

The identifier is generated when the possession factor is enrolled.

It is stored:

- In the Possession-Factor Metadata of the `.ws` file.
- On the enrolled possession factor together with the possession-factor data.

The Possession Factor ID is not secret.

Its purpose is to associate a removable device with the vault security
configuration for which it was enrolled and to allow implementations to reject
obviously unrelated possession factors before attempting normal
authentication.

The Possession Factor ID is replaced whenever a new possession factor is
enrolled.

It must not be treated as a substitute for the Possession Secret.

Possession of or knowledge of the Possession Factor ID does not satisfy the
possession requirement.

### 9.3 Possession Secret

The Possession Secret is not stored in the `.ws` file.

It is a uniformly random 256-bit secret stored on the enrolled possession
factor.

Normal authentication requires the Possession Secret corresponding to the
current Possession-Factor Metadata.

The Possession Secret is used together with the PDK to derive the NUK as
defined by the cryptographic design.

A new Possession Secret is generated whenever a replacement possession factor
is enrolled.

### 9.4 Hardware and Filesystem Identifiers

Hardware and filesystem identifiers are not cryptographic possession
credentials.

This includes information such as:

- USB device serial numbers.
- Vendor identifiers.
- Product identifiers.
- Filesystem UUIDs.
- Volume labels.
- Operating-system device paths.

An implementation may use such information as a non-security-critical hint for
device discovery or user-interface purposes.

Authentication must not depend on these values being unique, secret, stable,
or resistant to spoofing.

A device presenting matching hardware metadata without the correct Possession
Secret must not satisfy normal authentication.

### 9.5 Reserved Bytes

The three Reserved bytes are reserved for future format extensions.

For vault format version 1, all Reserved bytes must be zero.

Version 1 implementations must reject non-zero Reserved bytes.

### 9.6 Validation

Before possession-factor data is used for normal authentication, the
Possession-Factor Metadata must be structurally validated.

Validation must reject at least:

- Unsupported Factor Types.
- Invalid Reserved bytes.
- Missing or malformed Possession Factor IDs.
- Truncated Possession-Factor Metadata.

A matching Possession Factor ID only establishes that a device claims to
belong to the expected possession-factor configuration.

It does not establish successful authentication.

Cryptographic possession is established only when the secret obtained from the
device contributes to a NUK that successfully authenticates and decrypts the
current Normal VMK Wrapper.

### 9.7 Possession-Factor File

The enrolled removable storage device contains a dedicated possession-factor
file named `.wraithseal`.

The file contains the persistent information required to associate the device
with a vault and obtain the Possession Secret used for normal authentication.

For possession-factor format version 1, the file contains:

| Field                | Description                                                             |
| -------------------- | ----------------------------------------------------------------------- |
| Magic                | Identifies the file as WraithSeal possession-factor data.               |
| Format Version       | Identifies the possession-factor file format.                           |
| Vault ID             | Identifies the vault for which the factor was enrolled.                 |
| Security Generation  | Identifies the security generation for which the factor was enrolled.   |
| Possession Factor ID | Identifies the current possession-factor configuration.                 |
| Possession Secret    | Provides the 256-bit secret required by the normal authentication path. |

The `.wraithseal` file is stored at the root of the enrolled removable storage device.

The file name and its non-secret metadata are not security mechanisms.

The confidentiality of the Possession Secret must not depend on the file being
hidden by the filesystem or operating system.

### 9.8 Possession-Factor Association

When a removable device is discovered, the system may inspect its `.wraithseal`
file before attempting normal authentication.

The stored Vault ID must match the Vault ID of the vault being opened.

The stored Security Generation must match the Security Generation of the
current authoritative Security Metadata.

A generation mismatch indicates that the possession factor belongs to a
different security configuration and must prevent its use for normal
authentication.

The stored Possession Factor ID must match the identifier contained in the
current Possession-Factor Metadata.

A mismatch indicates that the device does not belong to the current
possession-factor configuration.

Matching identifiers do not establish authentication.

Only successful derivation of the NUK and authenticated decryption of the
Normal VMK Wrapper establish that the supplied Possession Secret is valid.

### 9.9 Possession-Factor File Validation

The `.wraithseal` file must be structurally validated before its contents are
used.

Validation must reject at least:

- Invalid magic values.
- Unsupported possession-factor format versions.
- Invalid Vault IDs.
- Invalid Security Generation values.
- Invalid Possession Factor IDs.
- Missing or malformed Possession Secrets.
- Truncated possession-factor files.
- Unexpected trailing data when not permitted by the format version.

A structurally invalid Security Generation, including the reserved value `0`,
must be rejected as malformed possession-factor data. A structurally valid
Security Generation that differs from the authoritative vault generation is an
association mismatch rather than a malformed file.

Failure to validate the `.wraithseal` file must prevent that device from being
used as the possession factor.

### 9.10 Possession-Factor File Layout

For possession-factor format version 1, the `.wraithseal` file has a fixed
serialized size of 88 bytes.

The layout is:

| Offset |     Size | Field                | Description                                         |
| -----: | -------: | -------------------- | --------------------------------------------------- |
| `0x00` |  8 bytes | Magic                | Identifies WraithSeal possession-factor data.       |
| `0x08` |  2 bytes | Format Version       | Identifies the possession-factor file format.       |
| `0x0A` |  2 bytes | File Size            | Total serialized file size.                         |
| `0x0C` |  4 bytes | Reserved             | Reserved for future format extensions.              |
| `0x10` | 16 bytes | Vault ID             | Identifies the associated vault.                    |
| `0x20` |  8 bytes | Security Generation  | Identifies the associated security generation.      |
| `0x28` | 16 bytes | Possession Factor ID | Identifies the enrolled possession factor.          |
| `0x38` | 32 bytes | Possession Secret    | Secret material required for normal authentication. |

### 9.11 Possession-Factor Magic

The first eight bytes of the `.wraithseal` file contain:

```text
57 53 46 41 43 54 00 00
```

which corresponds to:

```text
WSFACT\0\0
```

The magic value provides file-type identification only.

It does not authenticate the possession factor and must not be treated as
secret.

### 9.12 Possession-Factor Format Version

The Format Version identifies the byte-level interpretation of the
`.wraithseal` file.

For possession-factor format version 1:

```text
1
```

An implementation must reject unsupported possession-factor format versions.

The possession-factor Format Version is independent from the Vault Format
Version and Cryptographic Design Version.

### 9.13 Possession-Factor File Size

The File Size field contains the complete serialized size of the `.wraithseal`
file.

For possession-factor format version 1: `88`.

An implementation must reject a version 1 possession-factor file whose
serialized size does not match this value.

### 9.14 Possession-Factor Reserved Bytes

The four Reserved bytes are reserved for future format extensions.

For possession-factor format version 1, all Reserved bytes must be zero.

Version 1 implementations must reject non-zero Reserved bytes.

### 9.15 Vault Association

The Vault ID stored in the `.wraithseal` file must match the Vault ID stored in
the fixed header of the `.ws` vault.

A mismatch indicates that the possession-factor file belongs to another vault.

The Vault ID is not secret and does not establish authentication.

### 9.16 Security Generation

The `.wraithseal` file contains the Security Generation for which the
possession factor was enrolled.

The value must match the Security Generation of the authoritative Security
Metadata before the possession factor can be used for normal authentication.

A possession factor associated with an older or newer Security Generation must
not be accepted against the current security state.

The Security Generation is not secret and does not establish authentication.

It is used to associate the possession factor with the exact security
configuration for which it was created.

### 9.17 Factor Association

The Possession Factor ID stored in the `.wraithseal` file must match the
Possession Factor ID stored in the current Possession-Factor Metadata.

A mismatch indicates that the possession factor does not belong to the current
security configuration.

A matching identifier is only an association check and does not establish
cryptographic possession.

### 9.18 Possession Secret Storage

The final 32 bytes of the `.wraithseal` file contain the current Possession
Secret.

The Possession Secret is uniformly random 256-bit material generated using the
system CSPRNG.

It is the only secret value stored in the possession-factor file.

The Possession Secret must never be copied into the `.ws` vault file.

Because the possession factor is implemented using standard removable storage,
the `.wraithseal` file can be copied.

A copied `.wraithseal` file containing the current Possession Secret must be
treated as cryptographically equivalent to a cloned possession factor.

The security design therefore does not assume that standard removable storage
provides tamper resistance, non-exportability, or hardware-backed key
protection.

---

## 10. Encrypted Data Region

The Encrypted Data Region contains the encrypted representation of the vault
contents.

Vault contents are divided into independently authenticated fixed-size chunks
rather than being encrypted as a single monolithic ciphertext.

This design provides:

- Random access to encrypted vault data.
- Localized reads and writes.
- Independent authentication of individual chunks.
- Detection of chunk corruption or substitution.
- Credential rotation without re-encrypting vault contents.

For vault format version 1, the plaintext chunk size is:

```text
65536 bytes
```

corresponding to 64 KiB.

The Encrypted Data Region is logically structured as:

```text
Encrypted Data Descriptor
Chunk 0
Chunk 1
Chunk 2
...
Chunk N
```

### 10.1 Encrypted Data Descriptor

The Encrypted Data Descriptor contains the authenticated structural information
required to interpret the chunked data region.

Before encryption, the descriptor has a fixed size of 64 bytes.

Its plaintext layout is:

|     Size | Field               | Description                                           |
| -------: | ------------------- | ----------------------------------------------------- |
|  2 bytes | Data Region Version | Identifies the encrypted data-region format.          |
|  2 bytes | Descriptor Size     | Size of the plaintext descriptor.                     |
|  4 bytes | Chunk Size          | Plaintext size of each data chunk.                    |
|  8 bytes | Chunk Count         | Number of encrypted chunks in the data region.        |
|  8 bytes | Plaintext Length    | Logical length of the complete plaintext data stream. |
| 40 bytes | Reserved            | Reserved for future extensions.                       |

For vault format version 1:

```text
Data Region Version = 1
Descriptor Size     = 64
Chunk Size          = 65536
```

The Reserved bytes must be zero.

### 10.2 Descriptor Protection

The Data Descriptor is encrypted and authenticated using
XChaCha20-Poly1305 under the VMK.

Its serialized representation is:

|     Size | Field                 |
| -------: | --------------------- |
| 24 bytes | Descriptor Nonce      |
| 64 bytes | Descriptor Ciphertext |
| 16 bytes | Authentication Tag    |

The complete serialized descriptor therefore occupies `104 bytes`.

The Descriptor Nonce is generated using the system CSPRNG.

The descriptor associated data contains:

```text
wraithseal/data-descriptor/v1
Vault ID
Vault Format Version
Data Region Version
```

The associated data is encoded according to the canonical encoding rules
defined by this format.

The descriptor must be successfully authenticated before its contents are
trusted.

### 10.3 Chunk Structure

Each plaintext data chunk is exactly 65536 bytes before encryption.

Each chunk is independently protected using XChaCha20-Poly1305 under the VMK.

A serialized chunk has the following layout:

|        Size | Field              |
| ----------: | ------------------ |
|    24 bytes | Chunk Nonce        |
| 65536 bytes | Chunk Ciphertext   |
|    16 bytes | Authentication Tag |

The complete serialized size of a version 1 encrypted chunk is therefore `65576 bytes`.

The Chunk Nonce is generated independently whenever a chunk is encrypted or rewritten.

A nonce must never be intentionally reused with the same VMK.

### 10.4 Chunk Index

Chunks are numbered sequentially beginning with `0`.

The chunk index is determined by the position of the serialized chunk within
the Encrypted Data Region and is not stored as a duplicate field inside the
chunk.

The index participates in the authenticated associated data of the chunk.

Moving, reordering, or substituting a chunk must therefore cause
authentication failure.

### 10.5 Chunk Associated Data

Each encrypted chunk is bound to its position and vault through authenticated
associated data.

For vault format version 1, the chunk associated data contains:

```text
wraithseal/data-chunk/v1
Vault ID
Vault Format Version
Data Region Version
Chunk Index
```

The associated data does not include the Security Generation.

Credential rotation changes the protection around the VMK but preserves the
VMK itself. Binding data chunks to the Security Generation would therefore
require all vault contents to be re-encrypted whenever credentials are
rotated, which is intentionally avoided by the design.

### 10.6 Final Chunk and Padding

The final logical plaintext chunk may contain fewer than 65536 bytes of vault
data.

Before encryption, the unused portion of the final chunk is filled with zero
bytes so that every encrypted chunk has the same serialized size.

The exact logical plaintext length is stored in the authenticated Encrypted
Data Descriptor.

Padding bytes are not part of the logical vault contents.

When a final chunk is decrypted, bytes beyond the authenticated Plaintext
Length must be discarded.

### 10.7 Empty Data Region

A data region may contain zero chunks.

In this case:

```text
Chunk Count      = 0
Plaintext Length = 0
```

The authenticated Data Descriptor is still present.

### 10.8 Data Region Validation

After the VMK has been recovered, the Encrypted Data Descriptor must be
authenticated and validated before chunks are processed.

Validation must reject at least:

- Unsupported Data Region Versions.
- Invalid Descriptor Sizes.
- Unsupported Chunk Sizes.
- Non-zero Reserved bytes.
- Impossible Chunk Count and Plaintext Length combinations.
- A serialized region length inconsistent with the authenticated Chunk Count.
- Truncated chunk records.
- Additional chunk records not represented by the authenticated descriptor.

Each chunk must then be independently authenticated before its plaintext is
used.

Failure to authenticate any required chunk must be treated as encrypted data
corruption or integrity failure.

A failed chunk must never produce plaintext that is exposed as valid vault
contents.

---

## 11. Canonical Encoding

All persistent structures and cryptographic contexts use a deterministic
canonical byte representation.

Canonical encoding ensures that equivalent logical values always produce the
same byte sequence independently from the implementation language, operating
system, or processor architecture.

Implementations must not use platform-native structure serialization, memory
layout, padding, or endianness when producing persistent or cryptographic
representations.

### 11.1 Integer Encoding

All integer values are unsigned unless explicitly stated otherwise.

Multi-byte integers are encoded using big-endian byte order.

The following fixed-width integer representations are used:

| Type  |    Size | Range                         |
| ----- | ------: | ----------------------------- |
| `u8`  |  1 byte | `0` to `255`                  |
| `u16` | 2 bytes | `0` to `65535`                |
| `u32` | 4 bytes | `0` to `4294967295`           |
| `u64` | 8 bytes | `0` to `18446744073709551615` |

For example, the `u32` value `65536` is encoded as: `00 01 00 00`.

Variable-length integer encodings are not used in vault format version 1.

### 11.2 Fixed-Length Byte Fields

Fields such as Vault IDs, Possession Factor IDs, salts, nonces, keys,
ciphertexts, and authentication tags are represented as raw byte sequences of
the exact length defined by their containing structure.

No textual encoding such as hexadecimal or Base64 is applied when such values
are stored inside the `.ws`, `.wraithseal`, or `.wsr` binary formats.

For example, a 128-bit Vault ID occupies exactly 16 bytes.

### 11.3 Identifiers

Vault IDs and Possession Factor IDs are opaque 128-bit values.

Their canonical representation is the exact 16-byte value generated by the
system CSPRNG.

They are not serialized using textual UUID notation.

For example, separators, hexadecimal characters, braces, or UUID-specific
string formatting are not part of their persistent binary representation.

### 11.4 Domain Identifiers

Cryptographic domain identifiers are encoded as ASCII-compatible UTF-8 byte
strings.

A domain identifier is represented as:

```text
Domain Length (u16)
Domain Bytes
```

The Domain Length contains the number of bytes in Domain Bytes and does not
include the length field itself.

No terminating null byte is included.

For example:

```text
wraithseal/normal-unlock/v1
```

is encoded as:

```text
00 1B
77 72 61 69 74 68 73 65 61 6C 2F 6E 6F 72 6D 61
6C 2D 75 6E 6C 6F 63 6B 2F 76 31
```

Domain identifiers defined by the format are exact and case-sensitive.

### 11.5 Cryptographic Context Encoding

Cryptographic contexts are encoded as ordered sequences of fields.

Fields must appear in the exact order defined for the corresponding
cryptographic operation.

Implementations must not:

- Reorder fields.
- Omit fields.
- Insert implementation-specific fields.
- Serialize integer fields using platform-native byte order.
- Add separators or terminators that are not defined by the format.
- Use textual representations of binary values.

The domain identifier is always encoded first using the representation defined
in Section 11.4.

### 11.6 Normal Unlock Context

The Normal Unlock Context used by HKDF is encoded as:

| Order | Field                        | Encoding                      |
| ----: | ---------------------------- | ----------------------------- |
|     1 | Domain Identifier            | Length-prefixed domain string |
|     2 | Vault ID                     | 16 raw bytes                  |
|     3 | Security Generation          | `u64`                         |
|     4 | Cryptographic Design Version | `u16`                         |

The domain identifier is:

```text
wraithseal/normal-unlock/v1
```

The resulting byte sequence is supplied as the HKDF `info` value when deriving
the NUK.

### 11.7 Recovery Unlock Context

The Recovery Unlock Context uses the same structure as the Normal Unlock
Context:

| Order | Field                        | Encoding                      |
| ----: | ---------------------------- | ----------------------------- |
|     1 | Domain Identifier            | Length-prefixed domain string |
|     2 | Vault ID                     | 16 raw bytes                  |
|     3 | Security Generation          | `u64`                         |
|     4 | Cryptographic Design Version | `u16`                         |

The domain identifier is:

```text
wraithseal/recovery-unlock/v1
```

The resulting byte sequence is supplied as the HKDF `info` value when deriving
the RUK.

The distinct domain identifiers ensure cryptographic separation between the
normal and recovery derivation paths.

### 11.8 VMK Wrapper Associated Data

The associated data used to protect a VMK wrapper is encoded in the following
order:

| Order | Field                             | Encoding                         |
| ----: | --------------------------------- | -------------------------------- |
|     1 | Domain Identifier                 | Length-prefixed domain string    |
|     2 | Vault ID                          | 16 raw bytes                     |
|     3 | Vault Format Version              | `u16`                            |
|     4 | Metadata Version                  | `u16`                            |
|     5 | Cryptographic Design Version      | `u16`                            |
|     6 | Security Generation               | `u64`                            |
|     7 | Wrapper Type                      | `u8`                             |
|     8 | Password Derivation Configuration | Canonical 36-byte representation |
|     9 | Possession-Factor Metadata        | Canonical 20-byte representation |

The domain identifier is:

```text
wraithseal/vmk-wrapper/v1
```

The Wrapper Type is:

```text
0x01 = Normal VMK Wrapper
0x02 = Recovery VMK Wrapper
```

The same canonical representation must be reconstructed during authenticated
decryption.

Any difference in the encoded context must cause wrapper authentication to
fail.

### 11.9 Encrypted Data Descriptor Associated Data

The associated data used for the Encrypted Data Descriptor is encoded in the
following order:

| Order | Field                | Encoding                      |
| ----: | -------------------- | ----------------------------- |
|     1 | Domain Identifier    | Length-prefixed domain string |
|     2 | Vault ID             | 16 raw bytes                  |
|     3 | Vault Format Version | `u16`                         |
|     4 | Data Region Version  | `u16`                         |

The domain identifier is:

```text
wraithseal/data-descriptor/v1
```

The Security Generation is intentionally not included because the encrypted
data region remains valid across credential rotations that preserve the VMK.

### 11.10 Data Chunk Associated Data

The associated data for each encrypted data chunk is encoded in the following
order:

| Order | Field                | Encoding                      |
| ----: | -------------------- | ----------------------------- |
|     1 | Domain Identifier    | Length-prefixed domain string |
|     2 | Vault ID             | 16 raw bytes                  |
|     3 | Vault Format Version | `u16`                         |
|     4 | Data Region Version  | `u16`                         |
|     5 | Chunk Index          | `u64`                         |

The domain identifier is:

```text
wraithseal/data-chunk/v1
```

The Chunk Index begins at zero.

Binding the index into the authenticated context prevents a valid encrypted
chunk from being silently moved to another position within the same encrypted
data region.

The Security Generation is not included in chunk associated data.

### 11.11 Reserved Fields

Reserved fields are serialized exactly at the size defined by their containing
structure.

Unless explicitly defined otherwise, reserved fields in vault format version 1
must contain only zero bytes.

Reserved bytes participate in the serialized structure but must not be assigned
implementation-specific meaning.

### 11.12 Length and Offset Validation

Lengths and offsets must be decoded using their declared fixed-width integer
type before they are used.

Implementations must validate arithmetic involving offsets, lengths, and counts
before allocating memory or accessing the file.

Validation must detect at least:

- Integer overflow.
- Addition or multiplication overflow.
- Regions extending beyond the end of the file.
- Overlapping regions where overlap is not permitted.
- Lengths inconsistent with the corresponding format version.
- Chunk counts inconsistent with the Encrypted Data Region length.

A structurally invalid encoding must be rejected before cryptographic processing
continues with data derived from that structure.

---

## 12. Integrity and Authentication

The vault format uses authenticated encryption to detect unauthorized
modification of cryptographically protected data and security-critical
metadata.

Structural validation and cryptographic authentication are separate
operations.

Structural validation determines whether serialized data conforms to the
expected format.

Cryptographic authentication determines whether protected data and its
associated security context are authentic under the expected cryptographic key.

A structure that is syntactically valid must not therefore be considered
authentic solely because it passes structural validation.

### 12.1 Security Metadata Authentication

Security-critical metadata is bound to the VMK wrappers through authenticated
associated data.

For each VMK wrapper, the authenticated security context includes:

- Vault ID.
- Vault Format Version.
- Metadata Version.
- Cryptographic Design Version.
- Security Generation.
- Wrapper Type.
- Password Derivation Configuration.
- Possession-Factor Metadata.

The Password Derivation Configuration and Possession-Factor Metadata are
included using their complete canonical serialized representations.

Modification of any authenticated field must cause VMK-wrapper authentication
to fail.

The wrapper ciphertext and authentication tag are not included in their own
associated data.

### 12.2 Normal Wrapper Authentication

The Normal VMK Wrapper authenticates the current security configuration using
the NUK.

Successful authenticated decryption establishes that:

- The supplied NUK is valid for the protected VMK.
- The Normal VMK Wrapper has not been modified.
- The authenticated security context matches the context under which the
  wrapper was created.

Failure of authenticated decryption must not produce a usable VMK.

### 12.3 Recovery Wrapper Authentication

The Recovery VMK Wrapper authenticates the current security configuration using
the RUK.

Successful authenticated decryption establishes that:

- The supplied RUK is valid for the protected VMK.
- The Recovery VMK Wrapper has not been modified.
- The authenticated security context matches the context under which the
  wrapper was created.

Successful recovery-wrapper authentication does not authorize normal vault
access.

The recovered VMK remains restricted to the recovery workflow.

### 12.4 Encrypted Data Authentication

The Encrypted Data Descriptor and every encrypted data chunk are independently
authenticated using XChaCha20-Poly1305 under the VMK.

The Encrypted Data Descriptor authenticates the structural properties of the
encrypted data stream, including:

- Data Region Version.
- Chunk Size.
- Chunk Count.
- Plaintext Length.

Each data chunk authenticates:

- Its ciphertext.
- Its vault association.
- Its data-region version.
- Its position within the encrypted data stream.

A chunk must not be exposed as valid plaintext unless its authentication tag
has been successfully verified.

### 12.5 Truncation and Extension Detection

The authenticated Encrypted Data Descriptor defines the expected Chunk Count
and Plaintext Length.

After the descriptor has been authenticated, the implementation must verify
that the serialized Encrypted Data Region contains exactly the expected number
of chunks.

Removal of trailing chunks must therefore be detected.

Appending additional chunks must also be detected because they are not
represented by the authenticated Chunk Count.

A discrepancy between the authenticated descriptor and the physical data
region is an integrity failure.

### 12.6 Chunk Reordering and Substitution

Each data chunk is authenticated with its Chunk Index as part of its associated
data.

A valid chunk moved to another index must therefore fail authentication.

The Vault ID also participates in chunk associated data.

A chunk copied from another vault must therefore fail authentication even when
both vaults use the same format version.

### 12.7 Credential Rotation and Data Integrity

The Security Generation authenticates the current security configuration but
is intentionally excluded from encrypted data-chunk authentication.

Credential rotation preserves the VMK.

The Encrypted Data Region therefore remains cryptographically valid across:

- Password changes.
- Possession-factor replacement.
- Recovery-material rotation.
- Recovery.

These operations replace the protection around the VMK without requiring the
encrypted vault contents to be rewritten.

Security metadata from a previous generation must not authenticate as the
current security configuration.

This property does not prevent a complete historical vault snapshot from
remaining internally valid as its historical security generation.

### 12.8 Structural Integrity

Before cryptographic authentication is attempted, serialized structures must
be validated according to their format rules.

Structural validation includes:

- Magic values.
- Supported versions.
- Fixed field sizes.
- Reserved fields.
- Region offsets and lengths.
- Chunk counts.
- Required wrapper presence.
- Wrapper types.
- KDF parameter constraints.
- Possession-factor metadata structure.

Malformed structures must not be passed to cryptographic operations using
unchecked values.

### 12.9 Authentication Failure

Failure to authenticate protected vault data must fail closed.

An implementation must never:

- Return unauthenticated plaintext.
- Expose a VMK obtained from a failed wrapper operation.
- Continue processing a failed chunk as valid data.
- Silently ignore an invalid authentication tag.
- Replace authentication failure with unauthenticated recovery behaviour.

Authentication failures involving security metadata must be handled according
to the distinction between invalid credentials and corrupted persistent state
defined by the cryptographic design.

### 12.10 Authentication Scope

Authenticated encryption protects the integrity of the data and context covered
by each AEAD operation.

It does not by itself prevent:

- Deletion of the complete vault file.
- Replacement of the complete vault with an internally valid historical copy.
- Destruction of the possession-factor file.
- Destruction of recovery material.
- Modification of unauthenticated external filesystem metadata.

These conditions are outside the integrity guarantees provided by the vault
format itself.

---

## 13. Atomic Updates

Security-sensitive state transitions must not replace the current
authoritative Security Metadata in place.

The vault maintains two independently stored Security Metadata slots:

```text
Metadata Slot A
Metadata Slot B
```

At any point, one slot represents the current authoritative security
configuration while the other slot may contain an older generation or a newly
prepared candidate generation.

This design allows a new security configuration to be written and validated
without destroying the previously valid configuration.

### 13.1 Metadata Slots

The Fixed Header contains the offset and length of both Security Metadata
slots.

Each slot contains a complete Security Metadata structure as defined by this
format.

A slot must therefore contain all information required to represent one
complete security configuration, including:

- Metadata Header.
- Password Derivation Configuration.
- Possession-Factor Metadata.
- Normal VMK Wrapper.
- Recovery VMK Wrapper.

Security-critical fields must not be split across different generations or
different metadata slots.

### 13.2 Initial Vault State

When a vault is created, Security Generation begins at:

```text
1
```

One metadata slot contains the complete initial security configuration.

The other slot does not represent an authoritative security configuration
until a subsequent security-state transition requires it.

An unused slot must not be interpreted as valid Security Metadata.

### 13.3 Preparing a New Security State

A security-state transition from generation `N` to generation `N+1` is prepared
in the metadata slot that does not contain the current authoritative state.

The new slot is constructed completely before it may replace the current
security configuration.

Preparation includes:

- Creating the new security-generation value.
- Creating any new password derivation configuration required by the operation.
- Creating any new possession-factor metadata required by the operation.
- Deriving the required unlock keys.
- Creating both new VMK wrappers.
- Writing the complete Security Metadata structure.
- Validating the resulting serialized structure.

The authoritative generation `N` remains unchanged while generation `N+1` is
being prepared.

### 13.4 Candidate Validation

During an active security-state transition, a newly written metadata slot must
be read back and validated before the implementation reports the transition as
successful.

Validation includes:

- Structural validation of the complete metadata structure.
- Validation of all field lengths and versions.
- Validation of the expected Security Generation.
- Validation of the Password Derivation Configuration.
- Validation of the Possession-Factor Metadata.
- Presence of exactly one Normal VMK Wrapper.
- Presence of exactly one Recovery VMK Wrapper.
- Successful authentication of the newly created wrappers using the
  corresponding prepared authentication material.

A partially written or structurally invalid candidate slot must never
participate in authoritative-state selection.

If a complete higher-generation slot was durably written before an interruption,
there is no separate persistent marker recording whether runtime read-back and
wrapper validation completed. On the next open, authoritative-state selection
therefore follows Section 13.6. If the selected higher generation later fails
cryptographic authentication, the operation must fail closed and must not fall
back to a superseded generation.

### 13.5 Commit

A security-state transition becomes durably committed when the complete
generation `N+1` Security Metadata has been persistently written in a form that
will be selected as authoritative by the rules in Section 13.6.

Read-back validation is required before the implementation reports the
transition as successful, but the result of that validation is not represented
by a separate persistent commit marker.

External material required by the transition must be prepared before the new
Security Metadata becomes eligible to replace the current authoritative state.

The commit sequence is:

1. Recover the current VMK using the authorized security configuration.
2. Generate all new secrets and cryptographic material required by the transition.
3. Prepare and persist any required external possession or recovery material.
4. Construct the complete generation `N+1` Security Metadata.
5. Write generation `N+1` to the non-authoritative metadata slot.
6. Ensure that the new metadata slot has been persistently written.
7. Read and validate the complete new metadata slot.
8. Treat generation `N+1` as the current authoritative security configuration.

Generation `N` remains authoritative until the generation `N+1` metadata slot
has been completely and persistently written in a form that is eligible for
authoritative selection.

The current authoritative metadata slot must never be overwritten as part of
preparing generation `N+1`.

### 13.6 Authoritative Slot Selection

The authoritative security state is derived from the complete valid Security
Metadata slots stored in the vault.

When a vault is opened, both metadata slots must first undergo structural
validation.

A structurally invalid, incomplete, or unsupported slot is excluded from
authoritative-state selection.

If only one slot contains a complete valid security configuration, that slot is
the authoritative state.

If both slots contain complete valid security configurations, the slot with the
highest Security Generation is the authoritative state.

A lower valid generation is treated as superseded and must not be selected
while a complete valid higher generation exists.

The Fixed Header does not contain a mutable active-slot indicator.

This avoids requiring a separate persistent selector whose update would itself
need to be made atomic with the Security Metadata update.

Cryptographic authentication requiring user-supplied credentials is performed
against the selected current security configuration during authentication.

The selection of the highest complete generation protects against interrupted
metadata writes but does not provide protection against deliberate rollback of
the complete vault file.

### 13.7 Interrupted Update Recovery

If an interruption occurs while generation `N+1` is being written, the
previous generation `N` remains available in the other metadata slot.

When the vault is opened again:

- A malformed or incomplete candidate slot is rejected.
- A complete valid higher generation is selected.
- A valid previous generation remains usable when no complete higher
  generation exists.
- A partially written state must never be combined with fields from another
  metadata slot.

This behaviour ensures that interrupted security-state updates fail back to a
complete cryptographically consistent state.

### 13.8 Slot Reuse

After generation `N+1` becomes authoritative, the slot containing generation
`N` becomes the candidate slot for the next security-state transition.

For example:

```text
Generation 1 -> Slot A
Generation 2 -> Slot B
Generation 3 -> Slot A
Generation 4 -> Slot B
```

A slot may be overwritten only when the other slot contains the complete
current authoritative security configuration.

### 13.9 External Material

Some security-state transitions create or replace material stored outside the
`.ws` file.

External security material includes:

- The `.wraithseal` possession-factor file.
- Recovery material.

The `.ws` vault remains the authoritative representation of the current
Security Generation.

External material represents credentials associated with a particular vault
and security configuration but does not independently determine which Security
Generation is authoritative.

New external material required by a transition must be generated and
persistently written before the corresponding generation `N+1` Security
Metadata is written to the candidate metadata slot.

The generated external material must contain sufficient association information
to identify the vault and security configuration for which it was created.

Failure to create or persist required external material must abort the
transition before generation `N+1` becomes eligible to become authoritative.

If an interruption occurs after new external material has been written but
before generation `N+1` Security Metadata has been completely persisted, the
vault remains at generation `N`.

The newly created external material is then considered uncommitted material and
must not be accepted against generation `N`.

If generation `N+1` Security Metadata has been completely persisted and is
selected as authoritative, external material belonging only to generation `N`
must no longer satisfy the current authentication or recovery requirements.

### 13.10 Failure Behaviour

An interrupted or failed security-state transition must never produce a
security configuration assembled from multiple generations.

If neither metadata slot can be validated sufficiently to determine a complete
usable security state, the vault must enter an error condition.

The implementation must not silently reconstruct, guess, or partially accept a
security configuration whose authoritative state cannot be established.

### 13.11 Rollback Limitation

Dual metadata slots protect against incomplete local updates.

They do not provide protection against deliberate rollback of the complete
vault file.

An attacker capable of replacing the complete `.ws` file with an internally
valid historical copy may restore an older Security Generation together with
the security material that was valid for that historical state.

Preventing complete historical rollback requires authoritative state that
cannot itself be rolled back together with the vault file.

---

## 14. Recovery Material Format

Recovery material provides the external secret required by the recovery
authentication path.

It is stored separately from both the `.ws` vault and the enrolled possession
factor.

Recovery material is stored as a separate file using the `.wsr` extension.

For example:

```text
myvault.wsr
```

The recovery-material filename does not participate in authentication and does
not need to match the filename of the corresponding `.ws` vault.

Association with a vault is established through the Vault ID and Security
Generation stored inside the recovery material.

Renaming a `.wsr` file therefore does not alter its cryptographic meaning.

Recovery material is associated with exactly one vault and one Security
Generation.

It does not contain the VMK, PDK, RUK, password, or Possession Secret.

Possession of recovery material alone is insufficient to recover the vault.
Recovery additionally requires the correct password.

### 14.1 Recovery Material Contents

For recovery-material format version 1, the `.wsr` file contains:

| Field               | Description                                                       |
| ------------------- | ----------------------------------------------------------------- |
| Magic               | Identifies WraithSeal recovery material.                          |
| Format Version      | Identifies the recovery-material file format.                     |
| File Size           | Total serialized recovery-material file size.                     |
| Reserved            | Reserved for future format extensions.                            |
| Vault ID            | Identifies the vault for which the material was created.          |
| Security Generation | Identifies the security generation to which the material belongs. |
| Recovery Secret     | Provides the 256-bit secret required by the recovery path.        |

The Recovery Secret is the only secret value stored in the `.wsr` file.

The remaining fields provide structural information and associate the recovery
material with the vault and security configuration for which it was created.

### 14.2 Recovery Material Layout

For recovery-material format version 1, the `.wsr` file has a fixed serialized size of `72` bytes.

The layout is:

| Offset |     Size | Field               | Description                                           |
| -----: | -------: | ------------------- | ----------------------------------------------------- |
| `0x00` |  8 bytes | Magic               | Identifies WraithSeal recovery material.              |
| `0x08` |  2 bytes | Format Version      | Identifies the recovery-material file format.         |
| `0x0A` |  2 bytes | File Size           | Total serialized file size.                           |
| `0x0C` |  4 bytes | Reserved            | Reserved for future format extensions.                |
| `0x10` | 16 bytes | Vault ID            | Identifies the associated vault.                      |
| `0x20` |  8 bytes | Security Generation | Identifies the associated security generation.        |
| `0x28` | 32 bytes | Recovery Secret     | Secret material required for recovery authentication. |

The complete serialized size is therefore:

```text
8 + 2 + 2 + 4 + 16 + 8 + 32 = 72 bytes
```

All multi-byte integer fields use the canonical `big-endian` encoding.

### 14.3 Magic

The first eight bytes of recovery material contain:

```text
57 53 52 45 43 56 00 00
```

which corresponds to:

```text
WSRECV\0\0
```

The magic value provides file-type identification only.

It does not authenticate recovery material and must not be treated as secret.

### 14.4 Recovery-Material Format Version

The Format Version identifies the byte-level interpretation of recovery
material.

For recovery-material format version 1:

```text
1
```

An implementation must reject unsupported recovery-material format versions.

The recovery-material Format Version is independent from the Vault Format
Version, Metadata Version, and Cryptographic Design Version.

### 14.5 Recovery-Material File Size

The File Size field contains the complete serialized size of the recovery
material.

For recovery-material format version 1:

```text
72
```

An implementation must reject version 1 recovery material whose serialized
size does not match this value.

### 14.6 Reserved Bytes

The four Reserved bytes are reserved for future format extensions.

For recovery-material format version 1, all Reserved bytes must be zero.

Version 1 implementations must reject non-zero Reserved bytes.

### 14.7 Vault Association

The Vault ID stored in recovery material must match the Vault ID stored in the
Fixed Header of the `.ws` vault.

A mismatch indicates that the recovery material belongs to another vault and
must prevent its use for recovery.

The Vault ID is not secret and does not establish authentication.

### 14.8 Security Generation

Recovery material contains the Security Generation for which it was created.

The value must match the Security Generation of the authoritative Security
Metadata before the material can be used for recovery authentication.

Recovery material associated with an older or newer Security Generation must
not be accepted against the current security state.

The Security Generation is not secret and does not establish authentication.

It associates the Recovery Secret with the exact security configuration for
which it was generated.

### 14.9 Recovery Secret

The final 32 bytes contain the Recovery Secret.

The Recovery Secret is uniformly random 256-bit material generated using the
system CSPRNG.

It is used together with the PDK to derive the RUK as defined by the
cryptographic design.

The Recovery Secret must never be stored in the `.ws` vault or on the normal
possession factor.

The recovery material must never contain the VMK in either plaintext or
directly usable form.

Knowledge or possession of the Recovery Secret without the correct password
must not permit recovery.

### 14.10 Rotation and Invalidation

Recovery material belongs to exactly one Security Generation.

Whenever a security-state transition requires recovery-material rotation, a new
Recovery Secret is generated and new recovery material is created for
generation `N+1`.

After generation `N+1` becomes authoritative, recovery material belonging to
generation `N` must not authenticate against the current security state.

Old recovery material may remain physically present, but it is cryptographically
invalid for the current generation.

Rotation does not retroactively invalidate recovery material against a complete
historical copy of the vault from the generation to which that material
belonged.

### 14.11 Recovery Material During Security-State Transitions

New recovery material required by a security-state transition must be created
and persistently stored before the corresponding generation `N+1` Security
Metadata becomes authoritative.

The new material must not overwrite the only available copy of the recovery
material associated with generation `N` before generation `N+1` has been
successfully committed.

If an interruption occurs before generation `N+1` becomes authoritative, the
new recovery material is considered uncommitted and must not be accepted
against generation `N`.

If generation `N+1` becomes authoritative, the newly created recovery material
becomes the recovery material associated with the current security state and
the previous material becomes obsolete.

### 14.12 Validation

Recovery material must be structurally validated before its Recovery Secret is
used.

Validation must reject at least:

- Invalid magic values.
- Unsupported recovery-material format versions.
- Invalid file sizes.
- Non-zero Reserved bytes.
- Invalid Vault IDs.
- Invalid Security Generation values.
- Missing or malformed Recovery Secrets.
- Truncated recovery material.
- Unexpected trailing data.

A structurally valid Vault ID must match the vault being recovered.

A structurally valid Security Generation must match the authoritative Security
Generation.

Failure of either association check must prevent use of the Recovery Secret.

Matching association information does not establish successful recovery
authentication.

Recovery authentication succeeds only when the Recovery Secret, together with
the correct password, produces a RUK that successfully authenticates and
decrypts the current Recovery VMK Wrapper.

---

## 15. Validation and Failure Behaviour

The system must validate vault structures, external security material, and
cryptographic authentication results before allowing any operation that exposes
decrypted vault contents or modifies the current security state.

Validation failures must fail closed.

Invalid, incomplete, inconsistent, unsupported, or unauthenticated data must
never be interpreted as valid security state.

A validation failure must not cause the system to silently fall back to weaker
authentication requirements.

### 15.1 Validation Order

Validation should proceed from inexpensive structural checks to cryptographic
authentication.

For normal authentication, the validation sequence is:

1. Validate the Fixed Header.
2. Locate and structurally validate both Security Metadata slots.
3. Determine the authoritative Security Metadata structure according to the
   rules defined in Section 13.
4. Validate the Password Derivation Configuration.
5. Validate the Possession-Factor Metadata.
6. Validate the external `.wraithseal` file.
7. Verify that the Vault ID, Security Generation, and Possession Factor ID match
   the authoritative vault state.
8. Derive the PDK from the supplied password.
9. Derive the NUK using the PDK and Possession Secret.
10. Authenticate and decrypt the Normal VMK Wrapper.
11. Validate and authenticate the encrypted data structures before exposing
    plaintext data.

Failure at any step must terminate the normal authentication attempt.

For recovery authentication, the validation sequence is:

1. Validate the Fixed Header.
2. Locate and structurally validate both Security Metadata slots.
3. Determine the authoritative Security Metadata structure.
4. Validate the Password Derivation Configuration.
5. Validate the supplied `.wsr` recovery material.
6. Verify that its Vault ID and Security Generation match the authoritative
   vault state.
7. Derive the PDK from the supplied password.
8. Derive the RUK using the PDK and Recovery Secret.
9. Authenticate and decrypt the Recovery VMK Wrapper.

Failure at any step must terminate the recovery attempt.

Successful recovery authentication does not expose decrypted vault contents.
The VMK obtained through the recovery path may only be used to complete the
recovery transition defined by the architecture and cryptographic design.

### 15.2 Fixed Header Validation

Before using offsets or lengths stored in the Fixed Header, the implementation
must validate at least:

- The Magic value.
- The Vault Format Version.
- The Header Size.
- The Flags field.
- Reserved bytes.
- The Vault ID.
- Metadata slot offsets and lengths.
- The Encrypted Data Offset and Length.
- Integer ranges and arithmetic operations derived from these values.
- That referenced regions remain within the actual file boundaries.
- That regions do not overlap in ways prohibited by the format.

Unsupported Vault Format Versions must be rejected.

Malformed offsets or lengths must never be used to perform unchecked reads,
writes, allocations, or pointer arithmetic.

### 15.3 Security Metadata Validation

Each Security Metadata slot must be validated independently.

Structural validation must verify all fields required by the metadata format,
including:

- Metadata Version.
- Cryptographic Design Version.
- Security Generation.
- Password Derivation Configuration.
- Possession-Factor Metadata.
- Normal VMK Wrapper structure.
- Recovery VMK Wrapper structure.
- Reserved fields.
- Declared sizes and fixed-size requirements.

An incomplete or malformed metadata slot must not participate in authoritative
generation selection.

Fields from different metadata slots must never be combined to construct a
synthetic security state.

When both slots contain complete structurally valid metadata, the structure
with the highest Security Generation is selected according to Section 13.

### 15.4 External Material Validation

External possession and recovery material must be validated before their secret
values are used for key derivation.

For `.wraithseal` possession material, validation includes:

- File structure and exact serialized size.
- Magic and Format Version.
- Reserved fields.
- Vault ID.
- Security Generation.
- Possession Factor ID.
- Presence of the complete Possession Secret.

For `.wsr` recovery material, validation includes:

- File structure and exact serialized size.
- Magic and Format Version.
- Reserved fields.
- Vault ID.
- Security Generation.
- Presence of the complete Recovery Secret.

Structurally valid external material must still be rejected when its association
information does not match the authoritative vault state.

A filename, filesystem path, removable-device identifier, or storage location
must not substitute for these association checks.

### 15.5 Cryptographic Authentication Failures

Successful structural validation does not imply successful authentication.

AEAD authentication failure must be treated as authentication failure.

The implementation must not use plaintext produced by an unauthenticated or
failed AEAD operation.

A failed Normal VMK Wrapper authentication may indicate, among other causes:

- An incorrect password.
- Incorrect possession material.
- Corrupted authentication metadata.
- Corrupted wrapper data.
- Material associated with an incompatible security state.

A failed Recovery VMK Wrapper authentication may similarly indicate:

- An incorrect password.
- Incorrect recovery material.
- Corrupted authentication metadata.
- Corrupted wrapper data.
- Material associated with an incompatible security state.

The implementation is not required to expose the exact cryptographic cause of
authentication failure to the user.

### 15.6 Encrypted Data Validation

Possession of a successfully recovered VMK does not by itself establish that
the encrypted data region is valid.

Before plaintext is exposed, the implementation must validate the encrypted
data descriptor and the encrypted chunks according to Section 10.

Failure to authenticate the data descriptor must prevent the vault from being
opened.

Failure to authenticate any required encrypted chunk must prevent that chunk
from being accepted as valid plaintext.

The implementation must never silently return unauthenticated plaintext.

### 15.7 Unsupported Versions and Algorithms

The implementation must reject structures that require unsupported:

- Vault Format Versions.
- Metadata Versions.
- Cryptographic Design Versions.
- Recovery-material Format Versions.
- Possession-factor Format Versions.
- KDF algorithms.
- AEAD algorithms or parameters required by the encoded format.

Unsupported values must not be interpreted using the closest known version or
silently replaced with implementation defaults.

Version compatibility must be explicit.

### 15.8 Truncated and Extended Data

Fixed-size structures must have exactly the size required by their corresponding
format version.

Truncated structures must be rejected.

Unexpected trailing bytes in structures whose format requires an exact size
must be rejected unless the corresponding format version explicitly defines
them.

Variable-length regions must be validated using their encoded lengths and the
actual `.ws` file size before their contents are processed.

### 15.9 Integer and Boundary Validation

All decoded integer values must be validated before use.

The implementation must detect and reject conditions including:

- Integer overflow.
- Integer underflow.
- Offset addition overflow.
- Length calculations exceeding the vault file size.
- Invalid chunk counts.
- Invalid plaintext lengths.
- Impossible relationships between chunk count, chunk size, and plaintext
  length.
- Regions extending outside the vault file.
- Invalid or overlapping region boundaries.

Malformed input must not cause uncontrolled memory allocation or out-of-bounds
access.

### 15.10 Failure Atomicity

A failed validation or authentication attempt must not modify the authoritative
security state.

Security-state transitions must follow the atomic-update rules defined in
Section 13.

If preparation or validation of a candidate Security Generation fails, the
candidate must not replace the current authoritative generation.

Failure must not cause the system to combine partially updated metadata with
old or newly generated external material.

### 15.11 Runtime Failure Behaviour

A failure while unlocking must not transition the vault to `UNLOCKED`.

A failure while recovering must not expose decrypted vault contents.

A failure while sealing must not be reported as successful sealing unless the
system has actually completed the required sealing operations.

When the implementation cannot establish whether plaintext exposure has been
terminated successfully, it must not represent the vault as securely sealed.

Such conditions may transition the runtime state to `ERROR` according to the
architecture.

The `ERROR` state must not itself be interpreted as proof that the vault is
sealed.

### 15.12 Error Reporting

User-facing errors should provide enough information to support corrective
action without unnecessarily exposing sensitive internal state.

Authentication failures should not reveal derived keys, secret material,
cryptographic intermediate values, or decrypted wrapper contents.

Diagnostic information must not contain:

- Passwords.
- PDK values.
- NUK or RUK values.
- VMK values.
- Possession Secrets.
- Recovery Secrets.
- Decrypted vault contents.

Logs may identify structural or format errors where doing so does not disclose
secret material.

### 15.13 Fail-Closed Requirement

When the implementation cannot establish that a required security condition is
satisfied, the operation must fail.

In particular, the implementation must not:

- Bypass password authentication.
- Substitute recovery authentication for normal authentication automatically.
- Accept possession material from another vault or Security Generation.
- Accept recovery material from another vault or Security Generation.
- Ignore failed AEAD authentication.
- Ignore malformed security metadata.
- Select incomplete metadata as authoritative.
- Expose plaintext from unauthenticated encrypted data.
- Report a failed or uncertain sealing operation as successful.

Ambiguous security state must be treated as failure rather than successful
authentication, recovery, or sealing.

---

## 16. Glossary

| Term                                  | Definition                                                                                                                                            |
| ------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| **AAD**                               | Additional Authenticated Data. Non-encrypted data authenticated as part of an AEAD operation.                                                         |
| **AEAD**                              | Authenticated Encryption with Associated Data. Encryption providing confidentiality and integrity while authenticating additional non-encrypted data. |
| **Authoritative Security Metadata**   | The complete Security Metadata structure representing the current security state of the vault.                                                        |
| **Canonical Encoding**                | The deterministic byte representation used for serialized structures and cryptographic contexts.                                                      |
| **CSPRNG**                            | Cryptographically Secure Pseudorandom Number Generator. Used to generate cryptographic secrets, salts, nonces, and random identifiers.                |
| **Data Chunk**                        | A fixed-size unit of plaintext independently encrypted and authenticated within the Encrypted Data Region.                                            |
| **Data Descriptor**                   | The authenticated encrypted structure describing the logical organization of the Encrypted Data Region.                                               |
| **Encrypted Data Region**             | The region of the `.ws` file containing the encrypted data descriptor and encrypted data chunks.                                                      |
| **KDF**                               | Key Derivation Function. Argon2id is used for password-based derivation and HKDF-SHA-256 for unlock-key derivation.                                   |
| **NUK**                               | Normal Unlock Key. A 32-byte key derived from the PDK and Possession Secret and used to authenticate and decrypt the Normal VMK Wrapper.              |
| **Password Derivation Configuration** | The Argon2id parameters and salt required to derive the PDK from the user password.                                                                   |
| **PDK**                               | Password-Derived Key. A 32-byte intermediate key derived from the user password using Argon2id.                                                       |
| **Possession Factor**                 | The enrolled removable storage device providing the possession component required for normal authentication.                                          |
| **Possession Factor ID**              | A random 16-byte identifier associated with the enrolled possession factor. It is not secret.                                                         |
| **Possession Material**               | The `.wraithseal` file containing the Vault ID, Security Generation, Possession Factor ID, and Possession Secret.                                     |
| **Possession Secret**                 | A random 32-byte secret stored in the possession material and used together with the PDK to derive the NUK.                                           |
| **Recovery Material**                 | The `.wsr` file containing the Vault ID, Security Generation, and Recovery Secret required for recovery authentication.                               |
| **Recovery Secret**                   | A random 32-byte secret stored in recovery material and used together with the PDK to derive the RUK.                                                 |
| **RUK**                               | Recovery Unlock Key. A 32-byte key derived from the PDK and Recovery Secret and used to authenticate and decrypt the Recovery VMK Wrapper.            |
| **Security Generation**               | A monotonically increasing unsigned integer identifying the current security state of a vault.                                                        |
| **Security Metadata**                 | Security-sensitive vault metadata containing version information, password-derivation configuration, possession-factor metadata, and VMK Wrappers.    |
| **Security Metadata Slot**            | One of the two locations capable of storing a complete Security Metadata structure for atomic security-state updates.                                 |
| **Security-State Transition**         | An operation that changes security-sensitive material or configuration and advances the Security Generation.                                          |
| **Structural Validation**             | Validation of the syntax, sizes, ranges, and relationships of serialized data before it is trusted or used cryptographically.                         |
| **Vault**                             | The encrypted storage container represented by a single `.ws` file.                                                                                   |
| **Vault ID**                          | A random 16-byte identifier assigned when a vault is created and retained for its lifetime.                                                           |
| **VMK**                               | Vault Master Key. A random 32-byte key forming the cryptographic root for protecting vault contents.                                                  |
| **VMK Wrapper**                       | An authenticated encrypted representation of the VMK protected by either the NUK or RUK.                                                              |
| **XChaCha20-Poly1305**                | The AEAD construction used to protect VMK Wrappers and encrypted vault data.                                                                          |
