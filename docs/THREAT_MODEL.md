# WraithSeal Threat Model

## Table of Contents

1. [Introduction](#1-introduction)
2. [Security Objectives](#2-security-objectives)
3. [Assets](#3-assets)
4. [Trust Boundaries](#4-trust-boundaries)
5. [Adversary Model](#5-adversary-model)
6. [Security Assumptions](#6-security-assumptions)
7. [Threat Scenarios](#7-threat-scenarios)
8. [Threat Analysis](#8-threat-analysis)
9. [Out of Scope](#9-out-of-scope)
10. [Residual Risks](#10-residual-risks)

---

## 1. Introduction

This document defines the threat model for the encrypted vault system.

Its purpose is to identify the assets that require protection, the capabilities assumed for potential attackers, the trust boundaries of the system, and the security properties required to mitigate relevant threats.

The threat model focuses on the sealed state of the vault, the normal two-factor unlock process, possession-factor handling, and the recovery process.

The document does not define specific cryptographic algorithms or implementation mechanisms.

---

## 2. Security Objectives

The primary security objectives are:

- Preserve the confidentiality of vault contents while the vault is sealed.
- Preserve the integrity of vault contents and security-critical metadata.
- Prevent normal vault access unless both the correct password and valid possession factor are available.
- Prevent recovery unless both the correct password and valid recovery material are available.
- Ensure that recovery cannot bypass or replace the user's knowledge factor.
- Prevent invalidated possession factors from granting access.
- Prevent invalidated recovery material from completing recovery.
- Detect unauthorized modification of protected data or security-critical metadata.
- Minimize exposure of sensitive cryptographic material during normal operation.
- Fail securely when authentication, integrity validation, or sealing operations fail.

---

## 3. Assets

The system protects the following assets.

### 3.1 Vault Contents

User data stored inside the encrypted vault.

The confidentiality and integrity of this data must be preserved while the vault is sealed.

### 3.2 User Password

The knowledge factor used during normal authentication and recovery.

The password must not be stored or exposed in plaintext by the system.

The password is mandatory for both normal unlock and recovery. Recovery must not provide a mechanism for bypassing or replacing it.

### 3.3 Possession-Factor Material

Cryptographic material stored on or associated with the enrolled USB possession factor.

Possession of this material alone must not be sufficient to unlock the vault.

### 3.4 Recovery Material

High-entropy cryptographic material used during the recovery process.

Possession of recovery material alone must not be sufficient to recover the vault.

Recovery material substitutes only for the unavailable possession factor. It does not substitute for the user's password.

### 3.5 Cryptographic Keys

Any cryptographic key material used internally to protect vault data, authentication metadata, or recovery state.

### 3.6 Security-Critical Metadata

Metadata required to determine the state, validity, version, authentication configuration, or integrity of the vault.

Unauthorized modification of this metadata must not silently weaken the security properties of the system.

---

## 4. Trust Boundaries

The system operates across several trust boundaries.

### 4.1 Vault Storage

The encrypted vault container may be stored on media or systems that are not trusted to preserve confidentiality or integrity.

An attacker may be able to copy, replace, modify, or analyze the vault while it is sealed.

### 4.2 Host Operating System

The host operating system provides access to storage, removable devices, memory, and the user interface.

The system does not assume that an actively compromised host can be prevented from accessing plaintext data while the vault is unlocked.

### 4.3 USB Possession Factor

A standard USB storage device is not considered tamper-resistant hardware.

Its contents may potentially be copied, modified, or cloned by an attacker who gains sufficient access to the device.

### 4.4 Recovery Storage

Recovery material is expected to be stored separately from the normal possession factor and vault.

The storage location itself is not assumed to provide cryptographic protection.

### 4.5 Process Memory

Sensitive material required during authentication and vault access may temporarily exist in process memory.

The system must minimize its lifetime and exposure.

---

## 5. Adversary Model

The threat model assumes an attacker may have one or more of the following capabilities:

- Obtain a complete copy of the sealed vault.
- Perform unlimited offline analysis of copied vault data.
- Modify or replace the sealed vault container.
- Obtain the enrolled USB possession factor.
- Copy the contents of a standard USB possession factor.
- Obtain the recovery file.
- Attempt to use outdated possession or recovery material.
- Observe non-sensitive files, metadata, and application behaviour.
- Attempt repeated password guesses using material available offline.
- Interrupt, corrupt, or interfere with normal vault operations.

The attacker is not assumed to have automatically obtained the user's password merely by possessing the vault, USB device, or recovery file.

The attacker may possess multiple non-password factors simultaneously. The security model therefore requires the password to remain independently necessary for both normal unlock and recovery.

---

## 6. Security Assumptions

The threat model relies on the following assumptions:

- The host system is sufficiently trusted during vault unlock and use.
- The user is able to keep the password secret.
- The user is responsible for retaining the password because it cannot be recovered or bypassed by the recovery mechanism.
- Recovery material can be stored independently from the normal possession factor and vault.
- Established cryptographic primitives and maintained implementations behave according to their documented security properties.
- The operating system correctly enforces the storage and process isolation properties required by the implementation.
- The system has access to a cryptographically secure source of randomness.
- A standard USB device does not provide hardware-backed protection against cloning or direct extraction of its contents.

---

## 7. Threat Scenarios

### 7.1 Stolen Vault

An attacker obtains a complete copy of the sealed vault but does not possess the user's password, enrolled possession factor, or recovery material.

The attacker must not be able to access plaintext vault contents.

### 7.2 Stolen Vault and Password

An attacker obtains the sealed vault and learns the user's password but does not possess the enrolled possession factor or valid recovery material.

Normal vault access must remain unavailable.

Recovery must also remain unavailable without valid recovery material.

### 7.3 Stolen Vault and Possession Factor

An attacker obtains the sealed vault and the enrolled USB possession factor but does not know the user's password.

Normal vault access must remain unavailable.

### 7.4 Stolen Vault, Possession Factor, and Recovery Material

An attacker obtains the sealed vault, enrolled possession factor, and recovery material but does not know the user's password.

Neither normal unlock nor recovery must be possible.

### 7.5 Stolen Possession Factor

An attacker obtains or copies the enrolled USB possession factor without access to the user's password.

The possession factor alone must not reveal protected vault contents or sufficient information to independently derive access.

### 7.6 Cloned Possession Factor

An attacker creates a copy of the data stored on the enrolled USB device.

The security of the system must not depend on USB device identifiers being secret or impossible to reproduce.

A cloned possession factor must still require knowledge of the correct password before it can contribute to successful authentication.

### 7.7 Stolen Recovery File

An attacker obtains the recovery file but does not know the user's password.

Recovery must remain unavailable.

### 7.8 Stolen Vault and Recovery File

An attacker obtains both the sealed vault and recovery material but does not know the user's password.

Recovery must remain unavailable.

### 7.9 Offline Password Guessing

An attacker obtains sufficient stored material to test password guesses without interacting with the legitimate system.

The design must make offline password guessing computationally expensive enough to resist practical brute-force attacks against reasonable user passwords.

### 7.10 Vault Tampering

An attacker modifies the encrypted vault, authentication metadata, or other security-critical persistent data.

The system must detect unauthorized modification and must not expose corrupted plaintext as trusted data.

### 7.11 Use of Invalidated Possession Material

An attacker attempts to unlock the vault using possession material that was valid before a recovery or replacement operation.

The outdated material must no longer satisfy the possession requirement.

### 7.12 Use of Invalidated Recovery Material

An attacker attempts recovery using material that was valid before a previous successful recovery.

The outdated recovery material must no longer permit recovery.

### 7.13 USB Removal While Unlocked

The enrolled possession factor is removed while the vault is unlocked.

The system must attempt to seal the vault and must not report the vault as sealed unless the operation succeeds.

### 7.14 Interrupted Security Operation

An unlock, seal, recovery, possession-factor replacement, or recovery-material rotation operation is interrupted by application failure, system shutdown, storage failure, or unexpected device removal.

The resulting state must not silently weaken the security properties of the vault.

### 7.15 Lost Password

The user loses or forgets the vault password while still possessing the enrolled USB device, recovery file, or both.

The vault must remain inaccessible.

Neither the possession factor nor recovery material may bypass or replace the lost password.

Loss of the password therefore results in permanent loss of access to the vault.

### 7.16 Lost Possession Factor

The enrolled USB possession factor is lost, damaged, or otherwise unavailable while the user still possesses the correct password and recovery file.

The user must be able to recover access using the password and valid recovery material.

Following successful recovery, the previous possession material must be invalidated and a replacement possession factor may be enrolled.

### 7.17 Lost Recovery Material

The recovery file is lost while the user still possesses the correct password and enrolled USB possession factor.

Normal vault access must remain possible.

Loss of recovery material must not independently compromise the confidentiality of the vault, but it may remove the user's ability to recover from subsequent loss of the possession factor until valid recovery material is available again.

---

## 8. Threat Analysis

| Threat                                    | Attacker Capability                                                    | Required Security Property                                                       |
| ----------------------------------------- | ---------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Sealed vault theft                        | Copy the vault container.                                              | Vault contents remain confidential without valid authentication factors.         |
| Password compromise                       | Know the password and possess the vault.                               | Password alone cannot grant normal access or complete recovery.                  |
| USB theft                                 | Possess or copy the enrolled USB factor.                               | Possession material alone cannot grant normal access.                            |
| Recovery-file theft                       | Possess the recovery file.                                             | Recovery material alone cannot complete recovery.                                |
| Multiple non-password factors compromised | Possess the vault, USB factor, and recovery material.                  | The password remains independently required for both unlock and recovery.        |
| Offline password guessing                 | Analyze copied authentication material without rate limits.            | Password protection must resist practical offline brute-force attacks.           |
| Vault tampering                           | Modify persistent vault data or metadata.                              | Unauthorized changes are detected before protected data is trusted.              |
| USB cloning                               | Duplicate USB contents or identifiers.                                 | Security must not rely on uncloneable properties of standard USB storage.        |
| Outdated possession material              | Use a previously valid USB factor.                                     | Invalidated possession material cannot unlock the vault.                         |
| Outdated recovery material                | Use previously valid recovery material.                                | Invalidated recovery material cannot recover the vault.                          |
| Failed automatic seal                     | Prevent or interrupt sealing after USB removal.                        | The actual vault state remains explicit and is never falsely reported as sealed. |
| Lost password                             | Retain other authentication or recovery material without the password. | No alternative factor can bypass or replace the password.                        |
| Lost possession factor                    | Retain the password and recovery material.                             | Recovery can restore access without weakening the knowledge-factor requirement.  |
| Lost recovery material                    | Retain the password and possession factor.                             | Normal access remains possible without the recovery file.                        |

---

## 9. Out of Scope

The threat model does not attempt to provide complete protection against:

- A fully compromised operating system while the vault is unlocked.
- Malware with sufficient privileges to read decrypted vault data or process memory.
- Hardware keyloggers or other external mechanisms capable of directly capturing the user's password.
- Advanced physical attacks against system memory or hardware.
- Attacks against standard USB devices that assume tamper-resistant hardware properties.
- User coercion.
- Recovery from a forgotten or otherwise unavailable password.
- Loss of data caused by the absence of the required authentication and recovery material.
- Confidentiality of data that has already been copied outside the protected vault.

---

## 10. Residual Risks

Even when the system operates as intended, some risks remain.

A standard USB device can potentially be copied or cloned. The possession factor therefore provides a possession requirement but must not be treated as equivalent to a hardware security token.

While the vault is unlocked, plaintext data must be accessible to the host operating system. A compromised host may therefore read or modify unlocked data.

The security of password-based protection depends partly on the strength of the user's password and the effectiveness of the password-hardening mechanism.

Automatic sealing following USB removal may be delayed or prevented by operating-system conditions, open resources, application failure, or storage errors. The system must expose this condition rather than assuming the vault has been secured.

Loss of the user password makes the vault permanently inaccessible. Recovery does not provide a mechanism for bypassing or replacing the knowledge factor.

Loss of the enrolled possession factor together with loss of valid recovery material also makes the vault inaccessible, even if the user still knows the password.
