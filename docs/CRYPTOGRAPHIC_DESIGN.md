# WraithSeal Cryptographic Design

## Table of Contents

1. [Introduction](#1-introduction)
2. [Cryptographic Design Goals](#2-cryptographic-design-goals)
3. [Cryptographic Primitives](#3-cryptographic-primitives)
4. [Key Material](#4-key-material)
5. [Key Hierarchy](#5-key-hierarchy)
6. [Password-Derived Key](#6-password-derived-key)
7. [Normal Unlock Key](#7-normal-unlock-key)
8. [Recovery Unlock Key](#8-recovery-unlock-key)
9. [VMK Protection](#9-vmk-protection)
10. [Cryptographic Context and Binding](#10-cryptographic-context-and-binding)
11. [Security-State Transitions](#11-security-state-transitions)
12. [Sensitive Material Lifecycle](#12-sensitive-material-lifecycle)
13. [Cryptographic Failure Behaviour](#13-cryptographic-failure-behaviour)
14. [Glossary](#14-glossary)

---

## 1. Introduction

This document defines the cryptographic design used to protect encrypted vault
contents and security-critical key material.

It translates the security properties established by the project
specification, threat model, and system architecture into concrete
cryptographic constructions.

The design separates the encryption key protecting vault contents from the
credentials used to authorize access. Passwords, possession-factor secrets,
and recovery secrets do not directly encrypt vault contents. Instead, they are
used to derive key-protection material that protects the Vault Master Key
(VMK).

This separation allows authentication material to be replaced or rotated
without requiring the complete vault contents to be re-encrypted.

The design is based exclusively on established cryptographic primitives and
libraries. No custom cryptographic primitive is introduced.

This document defines cryptographic algorithms, key sizes, derivation
relationships, cryptographic contexts, and key-protection behaviour. It does
not define the final serialized vault format or operating-system-specific
implementation details.

---

## 2. Cryptographic Design Goals

The cryptographic design must preserve the authentication and recovery
properties defined by the system architecture.

In particular:

- Vault contents must be protected by high-entropy cryptographic key material.
- The user password must remain independently necessary for both normal access and recovery.
- Normal access must require both the password and the enrolled possession factor.
- Recovery must require both the password and valid recovery material.
- Possession-factor material alone must not provide access to the VMK.
- Recovery material alone must not provide access to the VMK.
- Possession-factor and recovery material together must not provide access without the password.
- Normal authentication and recovery authentication must use cryptographically separated derivation paths.
- Cryptographic material belonging to one vault must not become valid for another vault.
- Cryptographic material belonging to one security generation must not authenticate against a different current security generation.
- Protected key material must be cryptographically bound to its intended purpose and security context.
- Credential replacement and rotation should not require re-encryption of the complete vault contents.
- Cryptographic integrity failures must be detectable and must never be interpreted as successful authentication or decryption.
- Sensitive plaintext key material must exist only for as long as required by the operation using it.

---

## 3. Cryptographic Primitives

| Purpose                               | Primitive               |
| ------------------------------------- | ----------------------- |
| Password-based key derivation         | Argon2id                |
| Key derivation and factor combination | HKDF-SHA-256            |
| VMK protection                        | XChaCha20-Poly1305      |
| Random key and secret generation      | Operating-system CSPRNG |

### 3.1 Argon2id

`Argon2id` is used to transform the human password into fixed-length
cryptographic key material.

A password cannot be assumed to contain high entropy. The password derivation
step must therefore be deliberately expensive in memory and computation in
order to increase the cost of offline password guessing.

The output of Argon2id is the Password-Derived Key (PDK).

The PDK is 256 bits in length.

Each vault uses its own randomly generated password salt. The parameters
required to reproduce the derivation are stored as non-secret security
metadata.

These parameters include:

- Algorithm identifier.
- Salt.
- Memory cost.
- Time cost.
- Parallelism.
- Output length.

The complete Argon2id configuration required to reproduce the PDK must be
stored explicitly. Implementations must not rely on library defaults that may
change between software versions.

### 3.2 HKDF-SHA-256

`HKDF-SHA-256` is used to derive purpose-specific key material from existing
cryptographic secrets.

It is used to combine the PDK with the possession secret or recovery secret
while maintaining explicit separation between normal authentication and
recovery.

The normal and recovery paths use distinct cryptographic contexts.

HKDF output used as an unlock key is 256 bits in length.

### 3.3 XChaCha20-Poly1305

`XChaCha20-Poly1305` is used as an authenticated encryption with associated data
(AEAD) construction for protecting the VMK.

AEAD protection provides both:

- Confidentiality of the VMK.
- Authentication of the protected VMK and its associated cryptographic context.

The normal and recovery VMK wrappers are independently encrypted and use
distinct nonces.

A nonce must never be intentionally reused with the same encryption key.

### 3.4 Cryptographically Secure Random Generation

All high-entropy keys, secrets, salts, and nonces that require random
generation must be obtained from a cryptographically secure random source
provided by the operating system through the Platform Adapter or an
established cryptographic library backed by such a source.

---

## 4. Key Material

The cryptographic design uses several distinct forms of key and secret
material. Each has a specific purpose and lifecycle.

### 4.1 Vault Master Key

The Vault Master Key (`VMK`) is a 256-bit uniformly random secret generated
when the vault is created.

```text
CSPRNG -> VMK (32 bytes)
```

The VMK ultimately provides cryptographic access to protected vault contents.

It is independent from:

- The user password.
- The possession secret.
- The recovery secret.

The VMK is not regenerated merely because authentication credentials change.
Operations such as password changes, possession-factor replacement,
recovery-material rotation, and recovery normally replace the protection
around the VMK rather than re-encrypting all vault contents with a new VMK.

The VMK must never be stored persistently in plaintext.

### 4.2 Password-Derived Key

The Password-Derived Key (`PDK`) is a 256-bit value produced by applying
Argon2id to the user password.

```text
Password + Salt + KDF Parameters -> Argon2id -> PDK (32 bytes)
```

The PDK is an intermediate secret.

It is not itself sufficient to recover the VMK and must not be stored
persistently.

### 4.3 Possession Secret

The Possession Secret is a 256-bit uniformly random secret generated for an
enrolled possession factor.

```text
CSPRNG -> Possession Secret (32 bytes) -> Enrolled USB Device
```

The secret is stored on the enrolled removable device and contributes to the
normal unlock path.

Device serial numbers, filesystem identifiers, vendor identifiers, product
identifiers, and other hardware metadata are not substitutes for the
Possession Secret.

Because standard removable storage can be copied, possession of a copied
Possession Secret must be considered equivalent to compromise of that
possession factor.

### 4.4 Recovery Secret

The Recovery Secret is a 256-bit uniformly random secret generated for the
vault's current recovery configuration.

```text
CSPRNG -> Recovery Secret (32 bytes) -> Recovery Material
```

It is stored as part of the recovery material and contributes only to the
recovery authentication path.

The Recovery Secret does not replace the password.

Possession of the Recovery Secret without the correct password must not allow
the VMK to be recovered.

The VMK must never be stored directly in the recovery material.

### 4.5 Normal Unlock Key

The Normal Unlock Key (`NUK`) is a 256-bit derived key produced from:

- The Password-Derived Key.
- The current Possession Secret.
- A context identifying the normal unlock path and current vault security
  state.

The NUK protects the normal VMK wrapper.

It is transient key material and must not be stored persistently.

### 4.6 Recovery Unlock Key

The Recovery Unlock Key (`RUK`) is a 256-bit derived key produced from:

- The Password-Derived Key.
- The current Recovery Secret.
- A context identifying the recovery path and current vault security state.

The RUK protects the recovery VMK wrapper.

It is transient key material and must not be stored persistently.

---

## 5. Key Hierarchy

The VMK is the root secret providing access to encrypted vault contents.

Authentication credentials do not replace the VMK. Instead, they establish
two independent key-protection paths that can recover the same VMK for
different purposes.

The normal path is:

```text
Password -> Argon2id -> PDK
PDK + Possession Secret -> HKDF-SHA-256 -> NUK
NUK + Normal Wrapped VMK -> XChaCha20-Poly1305 -> VMK
```

The recovery path is:

```text
Password -> Argon2id -> PDK
PDK + Recovery Secret -> HKDF-SHA-256 -> RUK
RUK + Recovery Wrapped VMK -> XChaCha20-Poly1305 -> VMK
```

These paths are cryptographically separated.

The NUK must not be usable as the RUK, and the RUK must not be usable as the
NUK.

The VMK recovered through the normal path may be provided to the Vault Engine
for normal vault access after successful authentication.

The VMK recovered through the recovery path must only be used to establish a
new valid security configuration. Recovery must not use the recovered VMK to
expose decrypted vault contents.

---

## 6. Password-Derived Key

The password is processed independently from the possession and recovery
secrets.

For a vault with password salt `S_p`, the Password-Derived Key is:

```text
Password + S_p + KDF Parameters -> Argon2id -> PDK (32 bytes)
```

The password salt is not secret and is stored with the vault's security
metadata.

The salt must be generated using a CSPRNG when the password derivation
configuration is created.

A successful password change creates a new password derivation configuration
and new VMK wrappers while preserving the VMK itself.

Failure to derive the correct PDK must ultimately result in authentication
failure when protected key material is verified. The system must not require a
separate persistent password verifier if VMK-wrapper authentication provides
the required verification.

---

## 7. Normal Unlock Key

Normal authentication combines password-derived material with the secret
stored on the enrolled possession factor.

The derivation uses HKDF-SHA-256:

```text
PDK + Possession Secret + Normal Unlock Context -> HKDF-SHA-256 -> NUK
```

The Possession Secret therefore contributes directly to the derivation of the
key required to recover the normal VMK wrapper.

Neither input is sufficient independently:

| Available Material           | Normal VMK Recovery                           |
| ---------------------------- | --------------------------------------------- |
| Password only                | Not possible                                  |
| Possession Secret only       | Not possible                                  |
| Password + Possession Secret | Possible with valid current security metadata |

The Normal Unlock Context provides domain separation and binds the derived key
to its intended cryptographic purpose.

At minimum, the context identifies:

- The normal unlock purpose.
- The vault identity.
- The security generation.
- The cryptographic design version.

The context contains the equivalent of:

```text
wraithseal/normal-unlock/v1 + Vault ID + Security Generation
```

The final byte-level encoding of this context must be unambiguous and is part
of the persistent format definition.

---

## 8. Recovery Unlock Key

Recovery authentication combines password-derived material with the Recovery
Secret.

The derivation also uses HKDF-SHA-256, but with a distinct context:

```text
PDK + Recovery Secret + Recovery Unlock Context -> HKDF-SHA-256 -> RUK
```

Neither input is sufficient independently:

| Available Material         | Recovery VMK Recovery                         |
| -------------------------- | --------------------------------------------- |
| Password only              | Not possible                                  |
| Recovery Secret only       | Not possible                                  |
| Password + Recovery Secret | Possible with valid current security metadata |

The Recovery Unlock Context provides domain separation and binds the derived
key to the recovery path.

At minimum, the context identifies:

- The recovery purpose.
- The vault identity.
- The security generation.
- The cryptographic design version.

The context contains the equivalent of:

```text
wraithseal/recovery-unlock/v1 + Vault ID + Security Generation
```

The normal and recovery contexts must remain distinct so that key material
derived for one path cannot be interpreted as key material belonging to the
other.

---

## 9. VMK Protection

The same VMK is protected independently for normal authentication and
recovery.

For each security generation, the system maintains:

- A Normal Wrapped VMK.
- A Recovery Wrapped VMK.

Each wrapper is produced independently using XChaCha20-Poly1305.

### 9.1 Normal VMK Wrapper

```text
VMK + NUK + Normal Nonce + Normal Wrapper Context -> XChaCha20-Poly1305 -> Normal Wrapped VMK
```

The Normal Nonce must be generated according to the requirements of
XChaCha20-Poly1305 and must not be reused with the same NUK.

Successful authenticated decryption recovers the VMK and verifies that the
normal wrapper has not been modified or substituted outside its authenticated
context.

### 9.2 Recovery VMK Wrapper

```text
VMK + RUK + Recovery Nonce + Recovery Wrapper Context -> XChaCha20-Poly1305 -> Recovery Wrapped VMK
```

The Recovery Nonce must be independent from the Normal Nonce and must not be
reused with the same RUK.

Successful authenticated decryption recovers the VMK for recovery processing
only.

The recovered VMK must not be passed to the Vault Engine for normal data
access as part of the recovery workflow.

### 9.3 Wrapper Independence

The two wrappers protect the same VMK but belong to independent authentication
paths.

Compromise of one wrapped representation must not remove the authentication
requirements of the other path.

Replacing either authentication path creates a new wrapper for that path
without requiring the VMK itself to change.

---

## 10. Cryptographic Context and Binding

Cryptographic derivations and protected key material must be bound to their
intended vault, security generation, and purpose.

This prevents valid cryptographic material from being silently transplanted
between incompatible contexts.

### 10.1 Vault Binding

Key derivation and VMK protection must incorporate the identity of the vault.

Material created for one vault must therefore not become valid merely by being
copied into the security metadata of another vault.

### 10.2 Security-Generation Binding

Key-protection material is associated with a specific security generation.

After a security transition from generation `N` to generation `N+1`, material
from generation `N` must not authenticate against the current generation
`N+1` state.

This property does not retroactively invalidate complete historical copies of
generation `N`.

### 10.3 Purpose Binding

Normal unlock and recovery use distinct domain identifiers:

```text
wraithseal/normal-unlock/v1
wraithseal/recovery-unlock/v1
```

The corresponding VMK wrappers must likewise identify whether they belong to
the normal or recovery path.

### 10.4 Associated Data

VMK wrappers use authenticated associated data to bind encrypted key material
to non-secret security context.

The associated data must include at least:

- Vault identity.
- Security generation.
- Wrapper type.
- Vault format version.
- Cryptographic design version.

The wrapper type distinguishes at least:

- Normal VMK protection.
- Recovery VMK protection.

Associated data is authenticated but not encrypted. Any modification to
authenticated context must cause VMK unwrapping to fail.

The byte-level representation and canonical encoding of associated data are
defined as part of the persistent vault format.

---

## 11. Security-State Transitions

Security-sensitive credential changes modify the protection around the VMK
without replacing the VMK itself.

A transition creates a new complete security configuration before making it
authoritative.

The previous security generation remains current until the new configuration
has been fully prepared and validated.

A successful transition increments the security generation:

```text
Generation N -> Prepare New Security State -> Validate -> Atomic Commit -> Generation N+1
```

Material belonging to generation `N` must not satisfy authentication or
recovery requirements against the current state after generation `N+1` becomes
authoritative.

Historical copies of generation `N` remain cryptographically valid historical
states and cannot be retroactively altered by later rotations.

### 11.1 Password Change

A password change replaces the password-derived portion of both authentication
paths and rotates the recovery configuration.

The operation requires successful authentication using the current normal
authentication path before the password configuration can be replaced.

The current VMK is recovered using the existing valid normal authentication
configuration.

A new password salt is generated and the new Password-Derived Key is derived
from the new password:

```text
New Password + New Salt + KDF Parameters -> Argon2id -> New PDK
```

The current Possession Secret remains unchanged.

A new Recovery Secret is generated using a CSPRNG:

```text
CSPRNG -> New Recovery Secret (32 bytes)
```

New normal and recovery unlock keys are then derived for security generation
`N+1`:

```text
New PDK + Current Possession Secret + New Normal Context -> HKDF-SHA-256 -> New NUK
New PDK + New Recovery Secret + New Recovery Context -> HKDF-SHA-256 -> New RUK
```

The unchanged VMK is protected with new normal and recovery wrappers.

The new configuration includes:

- A new password salt.
- The selected Argon2id parameters.
- A new Recovery Secret.
- New recovery material.
- A new Normal Wrapped VMK.
- A new Recovery Wrapped VMK.
- New wrapper nonces.
- Security generation `N+1`.

The new recovery material must be fully generated and validated before the new
security state is committed.

After commit:

- The previous password must no longer recover the VMK against the current security generation.
- Previous recovery material must no longer authenticate against the current security generation.
- The newly generated recovery material becomes the only valid recovery material for the current vault state.

### 11.2 Possession-Factor Replacement

Possession-factor replacement changes the possession secret used by the normal
authentication path and rotates the recovery configuration.

The operation requires successful authentication using the current normal
authentication path.

The current VMK is recovered using the existing valid normal authentication
configuration and remains unchanged.

A new Possession Secret is generated using a CSPRNG:

```text
CSPRNG -> New Possession Secret (32 bytes)
```

The new Possession Secret is written to the replacement possession factor.

A new Recovery Secret is also generated:

```text
CSPRNG -> New Recovery Secret (32 bytes)
```

The new Recovery Secret is stored in newly generated recovery material.

New normal and recovery unlock keys are then derived for security generation
`N+1`:

```text
PDK + New Possession Secret + New Normal Context -> HKDF-SHA-256 -> New NUK
PDK + New Recovery Secret + New Recovery Context -> HKDF-SHA-256 -> New RUK
```

The unchanged VMK is protected with new normal and recovery wrappers.

The new security state includes:

- The new Possession Secret.
- Updated possession-factor metadata.
- A new Recovery Secret.
- New recovery material.
- A new Normal Wrapped VMK.
- A new Recovery Wrapped VMK.
- New wrapper nonces.
- Security generation `N+1`.

The replacement possession factor and new recovery material must be fully
generated and validated before the new security state is committed.

After commit:

- The replacement possession factor becomes the current enrolled factor.
- Previous possession material must no longer authenticate against the current security generation.
- The newly generated recovery material becomes the only valid recovery material for the current vault state.
- Previous recovery material must no longer authenticate against the current security generation.

### 11.3 Recovery-Material Rotation

Recovery-material rotation replaces the Recovery Secret without changing the
VMK or the current possession factor.

A new Recovery Secret is generated:

```text
CSPRNG -> New Recovery Secret (32 bytes)
```

The new Recovery Secret is stored in newly generated recovery material.

The VMK is recovered using the currently authorized normal path.

A new Recovery Unlock Key is derived:

```text
PDK + New Recovery Secret + New Recovery Context -> HKDF-SHA-256 -> New RUK
```

Because the security generation changes, the normal path must also be rebound
to generation `N+1`.

A new NUK is therefore derived using the existing Possession Secret and the new
normal context for generation `N+1`.

New normal and recovery VMK wrappers are then created.

The new security state includes:

- The new Recovery Secret.
- New recovery material.
- A new Normal Wrapped VMK.
- A new Recovery Wrapped VMK.
- New wrapper nonces.
- Security generation `N+1`.

The new recovery material must be fully generated and validated before commit.

After commit, previous recovery material must not satisfy recovery requirements
against the current security generation.

### 11.4 Recovery

Recovery is used when the current possession factor is unavailable but the user
still has:

- The correct password.
- Valid recovery material.

The password is processed using the current password derivation configuration:

```text
Password -> Argon2id -> PDK
```

The Recovery Secret is obtained from the recovery material.

The current Recovery Unlock Key is then derived:

```text
PDK + Recovery Secret + Recovery Context -> HKDF-SHA-256 -> RUK
```

The RUK is used to authenticate and recover the VMK from the current Recovery
Wrapped VMK.

Failure to authenticate the recovery wrapper terminates the recovery attempt.

Successful recovery of the VMK does not expose vault contents.

Instead, the system prepares a completely new security configuration.

A new Possession Secret and a new Recovery Secret are generated:

```text
CSPRNG -> New Possession Secret
CSPRNG -> New Recovery Secret
```

The new Possession Secret is enrolled onto the replacement possession factor,
and the new Recovery Secret is placed into newly generated recovery material.

New unlock keys are derived for security generation `N+1`:

```text
PDK + New Possession Secret -> HKDF-SHA-256 -> New NUK
PDK + New Recovery Secret -> HKDF-SHA-256 -> New RUK
```

The unchanged VMK is then protected with new normal and recovery wrappers.

The complete replacement configuration must be validated before it becomes
authoritative.

After a successful atomic commit:

- The security generation becomes `N+1`.
- The replacement possession factor becomes the current enrolled factor.
- The newly generated recovery material becomes the current recovery material.
- Previous possession material no longer authenticates against the current
  vault state.
- Previous recovery material no longer authenticates against the current vault
  state.
- The vault remains `SEALED`.

Temporary plaintext copies of the VMK, derived keys, old secrets, and other
sensitive intermediate material must be destroyed when they are no longer
required.

---

## 12. Sensitive Material Lifecycle

Cryptographic secrets must remain available in plaintext memory only for the
minimum duration required by the operation using them.

Sensitive material includes:

- The VMK.
- The PDK.
- The NUK.
- The RUK.
- Possession Secrets.
- Recovery Secrets.
- Temporary key-derivation output.
- Temporary plaintext produced during cryptographic operations.

Secrets must not be written to logs, diagnostic messages, crash reports, or
other non-secure output.

Persistent storage must contain protected representations or explicitly
required secret material only where the design requires it.

In particular:

- The VMK must never be stored persistently in plaintext.
- The PDK must never be stored persistently.
- The NUK must never be stored persistently.
- The RUK must never be stored persistently.
- The Possession Secret is stored on the enrolled possession factor.
- The Recovery Secret is stored in recovery material.

Where supported by the implementation platform and cryptographic library,
sensitive memory should be cleared explicitly after use.

Memory clearing is a best-effort defensive measure and must not be treated as
a guarantee that sensitive data has never existed elsewhere in process memory,
operating-system buffers, swap, crash dumps, or hardware-managed memory.

The design must therefore minimize:

- The number of copies of sensitive material.
- The lifetime of each copy.
- Transfers of sensitive material between components.
- Serialization of plaintext key material.

Components must receive only the sensitive material required for their
specific responsibility.

The Vault Engine, for example, may receive the VMK required to operate on vault
contents but must not receive the user password, Possession Secret, or Recovery
Secret.

---

## 13. Cryptographic Failure Behaviour

Cryptographic failure must always fail closed.

A failed cryptographic verification must never be interpreted as successful
authentication, successful decryption, or valid security metadata.

Failure conditions include:

- Argon2id derivation failure.
- HKDF derivation failure.
- Invalid or malformed cryptographic parameters.
- Missing cryptographic material.
- Invalid AEAD authentication tags.
- Invalid nonces or unsupported cryptographic configuration.
- Wrapper context mismatch.
- Vault identity mismatch.
- Security-generation mismatch.
- Wrapper-type mismatch.
- Corrupted protected key material.
- Failure of the operating-system CSPRNG.

### 13.1 Authentication Failure

If VMK unwrapping fails because authenticated decryption fails, the supplied
authentication material must be treated as invalid.

For a normal unlock attempt, authentication failure leaves the vault
`SEALED`.

For a recovery attempt, authentication failure terminates recovery and leaves
the vault `SEALED`.

The system must not expose whether a failure resulted specifically from:

- An incorrect password.
- Incorrect possession material.
- Incorrect recovery material.
- An invalid wrapper authentication tag.

Externally distinguishing these conditions could unnecessarily reveal
information about which authentication factor was correct.

### 13.2 Integrity Failure

Failure caused by corrupted or inconsistent persistent security state is
different from ordinary invalid authentication.

If security-critical metadata or authenticated context cannot be validated,
the system must not continue using that state as though it were valid.

Such failures must be surfaced as an integrity or vault-state error.

### 13.3 Random-Generation Failure

Failure of the cryptographically secure random source must abort any operation
that requires new cryptographic material.

The system must never substitute a weaker random generator or continue using
predictable fallback values.

This applies to generation of:

- VMKs.
- Password salts.
- Possession Secrets.
- Recovery Secrets.
- AEAD nonces where random generation is used.
- Any future cryptographic material requiring unpredictable randomness.

### 13.4 Security-State Transition Failure

A failed password change, possession-factor replacement, recovery-material
rotation, or recovery operation must not make a partial new configuration
authoritative.

Until the new security state has been completely prepared, validated, and
committed, generation `N` remains authoritative.

If failure occurs before commit:

- Newly generated wrappers must not replace current wrappers.
- Newly generated secrets must not become current credentials.
- The security generation must not advance.
- Temporary sensitive material must be discarded.
- The previous valid security state must remain current whenever it can be
  preserved safely.

If the implementation cannot determine which persistent security state is
authoritative after a failure, the vault must enter an error condition rather
than silently selecting one.

---

## 14. Glossary

| Term                             | Definition                                                                                                                                                  |
| -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Vault Master Key (VMK)**       | 256-bit random key material that ultimately provides cryptographic access to protected vault contents.                                                      |
| **Password-Derived Key (PDK)**   | 256-bit transient key derived from the user password using Argon2id.                                                                                        |
| **Normal Unlock Key (NUK)**      | 256-bit transient key derived from the PDK, Possession Secret, and normal unlock context. It protects the Normal Wrapped VMK.                               |
| **Recovery Unlock Key (RUK)**    | 256-bit transient key derived from the PDK, Recovery Secret, and recovery context. It protects the Recovery Wrapped VMK.                                    |
| **Possession Secret**            | 256-bit random secret stored on the enrolled possession factor and required for normal authentication.                                                      |
| **Recovery Secret**              | 256-bit random secret stored in recovery material and required together with the password for recovery.                                                     |
| **Normal Wrapped VMK**           | Authenticated encrypted representation of the VMK protected by the NUK.                                                                                     |
| **Recovery Wrapped VMK**         | Authenticated encrypted representation of the VMK protected by the RUK.                                                                                     |
| **Password Salt**                | Random non-secret value used as input to Argon2id for password-based key derivation.                                                                        |
| **AEAD**                         | Authenticated Encryption with Associated Data. Encryption that provides confidentiality and integrity while also authenticating non-secret associated data. |
| **CSPRNG**                       | Cryptographically Secure Pseudorandom Number Generator used to generate unpredictable cryptographic material.                                               |
| **Nonce**                        | Value required by an encryption construction to ensure safe use of a key across encryption operations.                                                      |
| **Associated Data**              | Non-secret information authenticated by an AEAD operation without being encrypted.                                                                          |
| **Domain Separation**            | Use of distinct cryptographic contexts so that key material intended for one purpose cannot be reused for another purpose.                                  |
| **Security Generation**          | Version of the current security configuration of a vault.                                                                                                   |
| **Cryptographic Design Version** | Identifier representing the cryptographic construction and interpretation expected by a vault security configuration.                                       |
| **Authoritative State**          | Security configuration currently recognized as valid and active for a vault.                                                                                |
| **Atomic Commit**                | Operation that makes a complete prepared security state authoritative without exposing a partially updated state.                                           |
