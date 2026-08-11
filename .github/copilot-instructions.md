# Kindling: Copilot Project Instructions

## 1. Project identity

**Kindling** is a Rust-based, cross-platform Kindle book-management application intended to run on **Linux and Windows**.

The initial real-world target devices are two **Amazon Kindle Paperwhite 12th generation (2024)** e-readers. The project began by proving the hardware and transport infrastructure against an actual Paperwhite before building application-level features or a GUI.

The user-facing application is called **Kindling**.

The core/device-management layer is called **Kindred**. Keep this naming distinction where it makes architectural sense:

- **Kindling** = application/UI/product.
- **Kindred** = Kindle discovery, identity, transport and device-management engine.

Do not force the word `kindred` into names where it makes code less clear, but preserve `kindred` as the core crate and device abstraction layer.

The GitHub repository is:

`Ken24T/kindling`

The repository is public. **Never commit private device identifiers, credentials, email addresses, Amazon account details, tokens or secrets.**

---

## 2. Primary product goal

Kindling should eventually provide a friendly desktop application for managing books and Kindle devices, including at minimum:

- Detect connected Kindle devices.
- Reliably distinguish multiple physical Kindles.
- Inspect device model and storage.
- Enumerate books and related content on the device.
- Transfer books to and from a device.
- Safely remove books from a device.
- Maintain a local book/library view independent of the attached device.
- Support more than one Kindle, initially the owner's and their spouse's Paperwhites.
- Run on Linux and Windows.

Future wireless capabilities are desirable, but should remain separate from the direct USB/MTP transport:

- **USB-C / MTP**: primary direct-management transport.
- **Wi-Fi**: potentially support Send to Kindle delivery and/or a local e-ink-friendly HTTP library from which a Kindle browser can download content.
- **Bluetooth**: recent Paperwhites have Bluetooth hardware, but stock Paperwhites expose it for audio/accessibility rather than a general-purpose file-transfer or management protocol. Do **not** treat Bluetooth as a supported Kindling data transport unless future evidence shows a usable supported mechanism.

Do not imply that Amazon exposes a documented LAN device-management API. Current architecture must not depend on one.

---

## 3. Hardware reference target

Infrastructure work has been tested against a real device with the following characteristics:

- Device: **Kindle Paperwhite, 12th generation**
- Firmware at initial development: **Kindle 5.19.5 (4794310058)**
- USB vendor/product ID: **`1949:9981`**
- USB manufacturer descriptor: **Amazon**
- USB product descriptor: **Kindle Paperwhite**
- Network capability reported by device: **Wi-Fi**
- Approximate raw MTP internal-storage capacity observed: **12.55 GB**
- Approximate free space observed during initial probe: **11.85 GB**

The device's full USB serial number is deliberately not documented in this public repository.

A crucial finding is that the **USB serial descriptor is stable across unplug/replug** and can be used as a persistent local device identity. The MTP device-info response on this model did **not** expose the serial number, so Kindred currently obtains identity through USB discovery rather than MTP.

Rules for device identity:

1. The stable USB serial is suitable as a local persistent key for distinguishing physical Kindles.
2. Never hard-code a user's serial number in source.
3. Store real serials only in local application configuration/database when that feature is implemented.
4. Mask serial numbers in ordinary logs and UI diagnostics unless the user explicitly requests the full value.
5. **Bus number and USB device address are transient and must never be used as persistent identity.** The device address changed after reconnect during testing, as expected.

---

## 4. Proven Paperwhite USB/MTP behaviour

The infrastructure has been tested directly on Linux, not inferred only from documentation.

### USB discovery

Linux `lsusb` identifies the Paperwhite as:

`1949:9981 Lab126, Inc. Kindle Paperwhite`

The USB descriptors expose:

- Manufacturer: Amazon
- Product: Kindle Paperwhite
- Persistent USB serial

Kindred reproduces this discovery directly in Rust using `rusb`.

### MTP behaviour

The 12th-generation Paperwhite presents its user storage through **MTP**, not as a conventional mounted USB mass-storage volume.

Diagnostic testing with `mtp-detect` showed:

- Manufacturer: Amazon
- Model: Kindle Paperwhite
- Friendly name: Kindle Paperwhite
- Storage description: Internal Storage
- Filesystem type: generic hierarchical
- Access capability: read/write
- MTP storage can be enumerated
- Folder and file enumeration matches the actual contents visible on the Kindle

The installed Linux `libmtp` 1.1.21 diagnostic tool reports VID/PID `1949:9981` as unknown in its static device list, but can still communicate successfully with the Kindle once the device interface is available. This is not considered a blocker.

Kindling itself is **not currently using libmtp**. The project instead uses the pure-Rust `mtp-rs` crate, which has successfully opened and interrogated the real Paperwhite.

`libmtp-dev`, `libmtp-runtime` and `mtp-tools` may exist on the development machine for diagnostics, but they are **not application dependencies** and should not become required merely because they were useful during investigation.

---

## 5. Important Linux MTP ownership issue

On Linux, desktop services may automatically claim the Kindle's MTP USB interface. During development on the reference machine, the process was:

`/usr/libexec/gvfsd-mtp`

When GVFS owns the interface, other MTP clients may fail with a device-busy or interface-claim error.

Useful diagnostic command:

```bash
ps -ef | grep -Ei '[g]vfsd-mtp|[k]iod[56]|[k]io.*mtp'
```

During development, the interface was temporarily released with:

```bash
pkill -x gvfsd-mtp
```

Do not assume killing GVFS is an acceptable product solution. Kindling should ultimately:

- Detect/report that the device is currently in use by another MTP client.
- Produce a clear user-facing message rather than a low-level USB panic/error.
- Avoid requiring root privileges.
- Investigate a clean desktop-integration strategy before shipping.

Also note that opening the Kindle in Dolphin or another file manager can cause the desktop MTP service to reclaim it.

---

## 6. Current Rust workspace

The repository is a Cargo workspace using resolver 3:

```text
kindling/
├── .github/
│   └── copilot-instructions.md
├── Cargo.toml
├── Cargo.lock
├── apps/
│   └── kindling-cli/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
└── crates/
    └── kindred/
        ├── Cargo.toml
        └── src/
            ├── lib.rs
            ├── usb.rs
            └── mtp.rs
```

Workspace members:

- `crates/kindred`
- `apps/kindling-cli`

The CLI is intentionally a diagnostic/test harness at this stage, not the final UI.

---

## 7. Current dependencies

At the time this instruction file was created:

### `kindred`

- `rusb = "0.9.4"`
- `mtp-rs = "0.30.0"`

### `kindling-cli`

- `futures = "0.3.34"`
- local path dependency on `kindred`

Rationale:

### `rusb`

Used for low-level USB device discovery and reading USB descriptors, including the stable serial number.

### `mtp-rs`

Used for MTP communication. It successfully opened the real Paperwhite and reported storage details. Prefer keeping MTP behind Kindred's own API rather than leaking `mtp-rs` types into application/UI code.

### `futures`

Currently used by the CLI only to `block_on` Kindred's async MTP probe without introducing a full async runtime prematurely.

Do not add Tokio or another large runtime solely because async code exists. Introduce runtime/framework dependencies only when there is a concrete application requirement.

---

## 8. Architecture rules

Maintain a strong separation between **device discovery**, **transport**, **Kindle semantics**, and the eventual **application/UI**.

Conceptually:

```text
Kindling UI / application
          |
          v
       Kindred
       /    \
      /      \
USB identity  MTP content transport
  (rusb)          (mtp-rs)
      \              /
       \            /
        Kindle Paperwhite
```

### USB discovery responsibilities

`kindred/src/usb.rs` currently handles:

- Enumerating USB devices.
- Filtering for Amazon VID `0x1949`.
- Recognising Paperwhite 12th gen PID `0x9981`.
- Reading manufacturer/product/serial descriptors where possible.
- Returning discovery even when optional string descriptors cannot be read.

Current public type/function include:

- `UsbKindle`
- `discover_kindles()`
- `AMAZON_VENDOR_ID`
- `PAPERWHITE_12_PRODUCT_ID`

Do not turn USB bus/device address into an identity mechanism.

### MTP responsibilities

`kindred/src/mtp.rs` currently handles a read-only probe:

- Open first available MTP device.
- Read MTP manufacturer/model.
- Enumerate storage.
- Read storage description, total capacity and free space.

Current public data types/function include:

- `MtpStorageSummary`
- `MtpProbe`
- `probe_first_mtp_device()`

The application/CLI should call Kindred abstractions rather than importing `mtp-rs` directly.

### Direction of travel

As the code grows, prefer a coherent transport/device abstraction rather than a collection of unrelated helper functions. However, do not prematurely create a large trait hierarchy before requirements justify it.

Eventually it may be useful to have concepts such as:

- device discovery/identity
- device profile
- transport/session
- Kindle inventory
- book/content object

Build these incrementally from observed requirements.

---

## 9. Milestones completed

### Milestone 0: repository/workspace bootstrap

Completed and committed.

- Empty GitHub repo cloned.
- Rust workspace created.
- `kindred` library crate created.
- `kindling-cli` binary crate created.
- Workspace builds cleanly.

Commit:

`94f030d Initialise Kindling Rust workspace`

### Milestone 1: Paperwhite USB discovery

Completed, tested against physical device, committed and pushed.

Proven behaviour from:

```bash
cargo run -p kindling-cli -- devices
```

The CLI successfully reported:

- Paperwhite 12th-generation model mapping.
- USB ID `1949:9981`.
- Amazon manufacturer.
- Kindle Paperwhite product descriptor.
- Masked stable serial.
- Current bus/device address.
- USB transport.

Commit:

`a3377d4 Add Paperwhite USB device discovery`

### Milestone 2A: read-only MTP probe

Completed, tested against physical device, committed and pushed.

Proven behaviour from:

```bash
cargo run -p kindling-cli -- mtp-probe
```

Successful result:

```text
MTP device opened
Manufacturer: Amazon
Model:        Kindle Paperwhite
Storage:      Internal Storage
Capacity:     12.55 GB
Free space:   11.85 GB
```

This is significant: **pure-Rust `mtp-rs` successfully communicates with the real 12th-generation Paperwhite.**

Commit:

`c8b77a1 Add read-only Paperwhite MTP probing`

At this checkpoint `main` was clean and synchronised with `origin/main`.

### Milestone 2B: read-only MTP root enumeration

Completed, tested against the physical Paperwhite, committed.

Proven behaviour from:

```bash
cargo run -p kindling-cli -- mtp-root
```

Observed output:

```text
Storage: Internal Storage

[DIR]  documents
[DIR]  screenshots
[FILE] FILE_SYSTEM_ACCESSIBILITY_FLAG
[DIR]  voice
[DIR]  system
[FILE] driveinfo.calibre
[DIR]  audible
[FILE] metadata.calibre
[DIR]  fonts
```

Key results:

- `documents` is identified as a directory through the Rust MTP layer — the milestone's core proof.
- The predicted calibre files (`metadata.calibre`, `driveinfo.calibre`) are present at the root; do not depend on them (§12).
- `FILE_SYSTEM_ACCESSIBILITY_FLAG` is present at the root; semantics not yet investigated — do not treat as a book.

### Milestone 2C: enumerate `documents/`

Completed, tested against the physical Paperwhite, committed.

Proven behaviour from:

```bash
cargo run -p kindling-cli -- mtp-documents
```

Observed output:

```text
Storage:        Internal Storage
Documents dir:  handle 4541

[DIR]  .cache
[DIR]  My Clippings.sdr
[DIR]  dictionaries
[DIR]  Downloads
```

Key results:

- The root `documents` folder was located (MTP handle 4541) and its immediate children enumerated.
- `My Clippings.sdr` confirms the `.sdr` sidecar-folder convention is present on this device — directly relevant to later delete-safety design (§11, Milestone 5).
- **Books are present but nested deeper than the `documents/` root.** `documents/Downloads/Items01/` (MTP handle 4625) holds **58 `.kfx` books**, each a `Title_ASIN.kfx` file paired with a `Title_ASIN.sdr` sidecar folder, plus `.yjf`/`.mf`/`.meta` per-book metadata. Dictionaries (`.azw`) live in `documents/dictionaries/`. This is the primary evidence base for the Milestone 3 inventory/format model — do not assume books sit at the `documents/` root.
- The storage root also contained `screenshots`, `voice`, `audible`, `system`, and `fonts` directories.
- The diagnostic `mtp-folder <handle>` command (Kindred `list_folder_children`) is what located the books; note that a libmtp `mtp-files` recursive scan under-reported them, while the Rust `mtp-rs` layer returned the complete folder listing.

Commit (2B + 2C together):

`a108ecc Add read-only MTP root and documents enumeration`

### Milestone 3: Kindle inventory model

Completed, tested against the physical Paperwhite, committed.

Proven behaviour from:

```bash
cargo run -p kindling-cli -- inventory
```

Result: **60 books inventoried** — 58 `.kfx` books + 2 `.azw` dictionaries. Each book lists title (from filename), format, size, ASIN and sidecar presence.

```text
Storage: Internal Storage
Books:   60

The Way of Kings
  KFX | 4.81 MB | asin: JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2
  sidecar: yes | metadata objects: 0
...
Oxford Dictionary of English
  AZW | 49.34 MB | asin: -
  sidecar: no | metadata objects: 0
```

Key results:

- `Book` model: ASIN key (10-char ASINs and 32-char hex ids both parse), title from filename, format (KFX/AZW), size; groups main file + `.sdr` sidecar + metadata handles.
- Book content is read from `documents/Downloads/Items01/`; dictionaries from `documents/dictionaries/`.
- First unit tests in the workspace (filename parsing, format mapping, sidecar/metadata grouping) — 6 passing.
- **Open question:** `.mf`/`.yjf`/`.meta` metadata objects were reported by libmtp under `Items01` but are NOT visible as direct children through `mtp-rs` `list_objects`. The model supports metadata association (unit-tested), but device-side association is unproven until a metadata milestone investigates.

Commit:

`1b5927e Add Kindle inventory model (Milestone 3)`

### Milestone 4: controlled transfer proof

Completed, tested against the physical Paperwhite, committed — with explicit write-milestone approval (2026-08-12).

Proven behaviour from:

```bash
cargo run -p kindling-cli -- mtp-write-test
```

Observed output:

```text
Controlled transfer test:
  folder:     kindling_m4_test_1786489330
  uploaded:   kindling_controlled_test.txt (34 bytes)
  readback:   verified
  cleaned up: yes
```

Key results:

- Upload, listing-verification, byte-for-byte readback and cleanup all succeed through pure-Rust `mtp-rs` on the physical Paperwhite.
- The test is tightly scoped: a dedicated `kindling_m4_test_<timestamp>` folder under `documents/` is created, filled with one 34-byte artifact, verified, then removed.
- Inventory before and after the test: **60 books, unchanged**; no leftover artifacts; storage root untouched.
- Kindred deliberately exposes exactly one write path (`run_controlled_transfer_test`) — no broad low-level write/delete primitives leak into the application layer.

Commit:

`81f6b89 Add controlled transfer proof (Milestone 4)`

### Milestone 5: safe device-management operations

Completed, tested against the physical Paperwhite, committed — with explicit approval for the controlled real-book removal (2026-08-12).

Kindred gains safe book-level operations in `manage.rs`:

- `copy_book_from_kindle(book, dest)` — download a book's content to a local path.
- `add_book_to_kindle(path)` — upload a local file to `documents/` (classic sideload location).
- `remove_book(book)` — validated, sidecar-aware removal: content file + `.sdr` sidecar folder + associated metadata. Every handle is checked against the device (expected name/kind) before deletion; nothing is deleted on mismatch.
- `remove_added_object(handle)` — the only handle-based delete; refuses anything whose filename does not begin with `kindling_` (controlled artifacts only).

Proven behaviour (physical device):

- `copy-book <asin> <dir>` — a real book was downloaded (201,255 bytes), filename preserved.
- `add-book <path>` — a controlled file uploaded to `documents/` and listed back.
- `remove-book <asin>` — "Moresome 3 Key West" was removed together with its `.sdr` sidecar: inventory 60 → 59, both objects gone from `documents/Downloads/Items01/`, siblings untouched.
- Guard — `remove-added` on a non-Kindling handle is refused (`access denied`).
- Restore — the removed book was re-added from its local backup via `add-book`; content is back on the device at `documents/`.

Note: the restored book lives in `documents/` root rather than `Downloads/Items01/`, so the M3 `inventory` command (which scans Items01 + dictionaries) reports 59 while the Kindle's own library scanner re-indexes sideloaded content from `documents/`. A future improvement is to have the inventory scan the whole `documents/` tree.

Commit:

`c0f6ae8 Add safe device-management operations (Milestone 5)`

---

## 10. Immediate next milestone

> **Status (2026-08-12):** Milestones 2B through 5 are complete — see §9 for the proven device output. The remaining roadmap is later product work per §11: the `.mf`/`.yjf` metadata milestone, the local library, error abstraction, device selection for multiple Kindles, and the GUI/async decisions.

### Milestone 2B: read-only MTP root enumeration

This is the next task unless the user explicitly redirects priorities.

Goal:

- Open the Paperwhite through Kindred.
- Enumerate the root of each MTP storage using `mtp-rs`.
- Return Kindred-owned summary types rather than exposing raw `mtp-rs` types to the CLI.
- Add a diagnostic command such as:

```bash
cargo run -p kindling-cli -- mtp-root
```

Expected style of output:

```text
Storage: Internal Storage

[DIR]  documents
[DIR]  system
[DIR]  fonts
...
```

The important proof is that Kindred can identify the `documents` folder through the Rust MTP layer.

Suggested internal types, subject to the actual `mtp-rs 0.30.0` API:

```rust
#[derive(Debug, Clone)]
pub struct MtpObjectSummary {
    pub filename: String,
    pub is_folder: bool,
}

#[derive(Debug, Clone)]
pub struct MtpStorageListing {
    pub description: String,
    pub objects: Vec<MtpObjectSummary>,
}
```

A likely high-level operation is `storage.list_objects(None).await`, where `None` denotes the storage root, but **verify the installed `mtp-rs 0.30.0` API rather than guessing field or method names**. Previous work already encountered a docs/API naming difference (`total_capacity` / `free_space` rather than guessed `*_bytes` fields), so compiler-guided verification is preferred.

Do not add write operations as part of 2B.

---

## 11. Planned milestones after 2B

### Milestone 2C: enumerate `documents/`

- Locate the MTP object handle for the root `documents` directory.
- Enumerate its immediate children.
- Determine how files, folders and Kindle-side auxiliary objects are represented.
- Keep this read-only.

Do **not** immediately label every object in `documents` as a book. First inspect what the device actually contains.

### Milestone 3: Kindle inventory model

Once MTP object behaviour is understood:

- Introduce Kindle-aware inventory types above raw MTP object summaries.
- Recognise supported book/document formats based on evidence from actual files.
- Consider associated metadata/cover/sidecar objects carefully.
- Separate raw storage path/object IDs from user-facing book identity.

### Milestone 4: controlled transfer proof

Only after read-only inventory is reliable:

- Upload one harmless controlled test file.
- Verify it appears correctly.
- Download/read back if useful.
- Remove the exact controlled test object.

Treat this as a tightly scoped integration test against the physical device.

### Milestone 5: safe device-management operations

Introduce application-level operations such as:

- add/import book
- copy to Kindle
- copy from Kindle
- remove book

Prefer safe Kindle-aware methods such as `remove_book(...)` over exposing arbitrary low-level delete primitives broadly through the application.

Deletion must include validation and should account for related metadata/sidecar/thumbnail objects if required.

### Later product work

Potential future areas include:

- local library database
- metadata and covers
- multiple named device profiles
- storage usage display
- search/filter/sort
- cross-platform GUI
- Windows device testing
- Send to Kindle integration where practical
- local Wi-Fi HTTP library for Kindle browser access
- firmware/version compatibility tracking

Do not select the GUI framework prematurely. Make the core/device boundary stable first.

---

## 12. Safety and device-integrity rules

The Kindle contains real user data. Treat physical-device operations conservatively.

Until explicitly approved for a write milestone:

- Prefer read-only MTP operations.
- Do not upload files.
- Do not delete files.
- Do not move or rename device objects.
- Do not modify Kindle system folders.
- Do not modify calibre metadata files merely because they exist.
- Do not attempt jailbreak-specific behaviour.

Before any destructive/device-changing operation:

1. Explain exactly what object/path will change.
2. Keep scope to a controlled test artifact wherever possible.
3. Ensure code cannot accidentally target the storage root or broad directory tree.
4. Make failures explicit and recoverable.

The Kindle root has been observed to include folders such as `documents` and `system`, and may include calibre-created files such as `metadata.calibre` and `driveinfo.calibre`. Kindling must not depend on calibre's private management files unless there is a separately justified interoperability requirement.

Kindle Collections are a separate and potentially difficult concern. Do not assume collections can be managed cleanly through ordinary USB/MTP file operations.

---

## 13. Cross-platform expectations

The product target is **Linux and Windows**, even though early hardware development is occurring on Linux.

Requirements:

- Avoid Linux-only assumptions leaking into domain/application APIs.
- Keep platform-specific device ownership/discovery details behind Kindred.
- Prefer dependencies with credible Windows support.
- `mtp-rs` was deliberately chosen partly because it offers a cross-platform MTP path rather than requiring Kindling to be fundamentally tied to Linux `libmtp`.
- Do not design UI or storage paths around POSIX-only conventions.
- Use Rust path types (`Path`, `PathBuf`) rather than constructing platform paths manually.

Linux-specific workarounds such as `pkill gvfsd-mtp` are development diagnostics, not product architecture.

Windows support is not yet hardware-tested in this repository. Clearly distinguish proven Linux behaviour from planned Windows behaviour.

---

## 14. Coding style and implementation approach

Prefer simple, maintainable Rust over clever abstractions.

General rules:

- Rust edition 2024.
- Format with `cargo fmt`.
- Keep warnings clean where practical.
- Use clear domain names and small cohesive modules.
- Return structured errors/results rather than printing from library code.
- Keep user-facing/diagnostic output in the CLI or future UI layer.
- Avoid panics for ordinary hardware/disconnection/permission failures.
- Treat device disappearance during an operation as a normal error case.
- Keep external-crate types behind Kindred APIs when doing so preserves portability and abstraction.
- Do not add large dependencies without a concrete reason.
- Prefer compiler-verified API usage over guessing from stale documentation.

Before considering a code change complete, normally run:

```bash
cargo fmt
cargo check
```

As tests become available, also run:

```bash
cargo test
```

When useful, run:

```bash
cargo clippy --workspace --all-targets
```

Do not claim an operation works on real hardware until it has actually been run against the physical Paperwhite and its output has been observed.

### Modularity rules (read first)

AI agents tend to accrete code into existing files. Resist that. These limits mirror `.github/TCTBP.json` → `developmentPolicy` (the machine-readable source of truth):

- **250 lines:** soft ceiling. When a file approaches this, ask whether the new code belongs in a new module.
- **400 lines:** warning threshold. Files above this require a comment at the top justifying why they haven't been split.
- **600 lines:** hard split. Files at or above this MUST be split before the next checkpoint. No exceptions.
- **Functions:** keep under 40 lines. Prefer pure functions; side effects should be explicit and documented.
- **Modules:** split when a group of 3+ related functions emerges, or when a single module accumulates many responsibilities. One clear purpose per file — if you need "and also" to describe it, split it.
- **Naming:** snake_case file/module names that match the crate layout (`usb.rs`, `mtp.rs`). Prefer small cohesive modules over large multi-purpose files.

### Critical rules

1. **Never commit secrets or identifiers** — no USB serials, credentials, tokens, or Amazon account details. Mask stable identifiers in logs and UI diagnostics (§18).
2. **Never perform destructive device operations without explicit approval** — no uploads, deletes, moves, or renames until the write milestone is explicitly approved (§12).
3. **No speculative changes** — do not refactor unrelated working infrastructure gratuitously, and do not "fix" things not mentioned in the request.
4. **Read before writing** — inspect existing code and understand the surrounding module before changing it.
5. **Keep Kindred's boundary intact** — application/CLI code calls Kindred abstractions; do not leak `mtp-rs` or `rusb` types into the application/UI layer (§8).

### Branch naming (short-lived task branches)

Short-lived task branches are created off `development` and closed via `branch <name>` (§22). Use conventional prefixes:

- `feature/<name>` — new capability
- `fix/<name>` — bug fix
- `chore/<name>` — maintenance/tooling
- `docs/<name>` — documentation
- `infrastructure/<name>` — CI/build/tooling

These prefixes align with the TCTBP profile's `minorBranchPrefixes` (`slice/`, `feature/`) and the CI trigger list in §22.

### Testing conventions

There are no tests in the workspace yet; these conventions apply from the first test written onward:

- Unit tests: `#[cfg(test)]` modules co-located with the code they test.
- Integration tests: `tests/` at the crate root (e.g. `crates/kindred/tests/`).
- The TCTBP `cargo test` gate is blocking for promote/ship (§22); keep it green before checkpointing.

### Interaction conventions

- **"Thoughts mode":** if a prompt ends with the word `thoughts`, respond with opinion and analysis only — no code or file changes.

---

## 15. CLI role

`kindling-cli` currently exists as an infrastructure and diagnostic harness.

Current commands:

- `devices` — list supported connected Kindle devices through Kindred USB discovery.
- `mtp-probe` — open the first MTP device through Kindred and report model/storage information.

The CLI should remain thin. It should:

- call Kindred APIs
- format results
- expose useful diagnostic commands

It should **not** become the owner of USB/MTP implementation logic.

The diagnostic CLI is expected to remain useful even after a GUI exists, particularly for troubleshooting firmware/device issues.

---

## 16. Device identity and profiles (resolved 2026-08-12)

Kindling serves two users, each on their own machine with their own Kindle: Ken on Linux, Deb on Windows. Only **one device is ever attached at a time**, so `MtpDevice::open_first()` remains the correct transport approach — there is no multi-device selection requirement.

What IS required is **device identity labelling**: when a Kindle is attached, Kindling must know whose it is, because a device might occasionally be plugged into the other person's machine. The stable USB serial (Milestone 1) is the persistent key; a **local JSON profile store** (`profiles.json`) maps serial → friendly name ("Ken's Kindle", "Deb's Kindle").

- Each machine keeps its own local profile store; the JSON file can be exchanged between machines so a cross-over plug-in is recognised rather than shown as unknown.
- The friendly name is a local profile mapping — not assumed to be provided by MTP.
- On an unprofiled device, Kindling reports "Unknown Kindle" and offers `profile add <name>`.
- Serials are stored locally only and masked in logs/UI (§18).

The profile store and the planned JSON local library (see PLAN.md) are the natural home for this identity data.

---

## 17. Error-handling expectations

Hardware applications must expect imperfect conditions. Design for at least:

- no Kindle connected
- unsupported Kindle model
- Kindle disconnected mid-operation
- USB descriptor unavailable
- MTP interface owned by another application/service
- access/permission denied
- storage full
- MTP operation timeout
- malformed/unexpected object metadata
- firmware/device behaviour changes

Translate low-level errors into useful Kindred/application errors when enough information exists.

Do not hide the original diagnostic context entirely. A friendly UI message and a useful debug/log cause can coexist.

---

## 18. Privacy and logging

The repository is public and will eventually process personal libraries.

Rules:

- Never commit real USB serial numbers.
- Never commit Amazon credentials, SMTP credentials or Send-to-Kindle account details.
- Mask stable identifiers in normal logs.
- Avoid logging entire private library inventories by default.
- Treat book titles/authors as user data in production logging.
- Keep device-profile data local unless an explicit sync feature is later designed.

---

## 19. Git/workflow expectations

Development so far has used small, working milestones committed directly to `main` after successful local checks and physical-device verification.

Existing commits in order:

1. `94f030d Initialise Kindling Rust workspace`
2. `a3377d4 Add Paperwhite USB device discovery`
3. `c8b77a1 Add read-only Paperwhite MTP probing`
4. `f49a6f8 Add Copilot project instructions and infrastructure handover`

When assisting inside VS Code:

- Inspect existing code before changing it.
- Make focused changes aligned to the current milestone.
- Do not refactor unrelated working infrastructure gratuitously.
- Do not push, create PRs, publish releases or make other external GitHub changes unless the user explicitly requests it.
- Before proposing a commit, ensure the working tree changes match the milestone and checks have passed.
- Prefer descriptive milestone-oriented commit messages.

---

## 20. Infrastructure facts already learned: do not rediscover unless needed

The following questions have already been answered experimentally:

- **Can Linux detect the Paperwhite over USB?** Yes.
- **Can Rust detect it through `rusb`?** Yes.
- **Is `1949:9981` the tested Paperwhite USB ID?** Yes.
- **Can the physical Kindle be persistently distinguished?** Yes, using the stable USB serial descriptor stored locally and masked in logs.
- **Is the bus/device address stable?** No.
- **Does the 12th-gen Paperwhite expose writable hierarchical MTP storage?** Yes.
- **Can `mtp-rs 0.30.0` open the tested device on Linux?** Yes.
- **Can it read storage description/capacity/free space?** Yes.
- **Does GVFS sometimes take exclusive ownership of the MTP interface?** Yes.
- **Is Bluetooth currently a viable book-management transport?** No evidence supports that.
- **Should Kindling depend on system `libmtp`?** Not currently; `mtp-rs` is the chosen application MTP layer.

Do not replace proven working infrastructure with another stack without a concrete technical benefit and explicit discussion.

---

## 21. Current checkpoint for the next coding session

At the time this file was created:

- `main` contains the four commits listed above.
- USB discovery is working on physical hardware.
- The MTP probe is working on physical hardware.
- The working direction is to move normal coding into VS Code while keeping infrastructure/architecture decisions deliberate and milestone-based.
- The **next coding task is Milestone 2B: read-only MTP root enumeration**.

Current state (2026-08-12):

- Milestones 2B and 2C are complete, committed as `a108ecc` on `development`, and published to origin (see §9).
- Root and `documents/` enumeration are proven on the physical Paperwhite.
- The repo runs the TCTBP staged branch model (§22); work continues on `development`.
- The Milestone 3 planning session is complete; the agreed scope is recorded in `PLAN.md` (root).
- Milestones 3, 4 and 5 are complete — see §9 (inventory model, controlled transfer proof, and safe device-management operations).
- **Next step:** the local profile registry and device identity slice is complete (§16, `profile.rs`); the JSON local library comes next, then the `.mf`/`.yjf` metadata milestone and error abstraction. The repo is at a stable checkpoint on `development`.

A good next sequence is:

1. Inspect `crates/kindred/src/mtp.rs`, `lib.rs` and CLI `main.rs`.
2. Verify the actual `mtp-rs 0.30.0` API for root object enumeration.
3. Add Kindred-owned object/listing summary types.
4. Implement a read-only function to list storage-root objects.
5. Export it from `kindred`.
6. Add `kindling-cli mtp-root` as a thin diagnostic command.
7. Run `cargo fmt` and `cargo check`.
8. Ensure `gvfsd-mtp` is not holding the device before the physical test.
9. Run the command against the Paperwhite.
10. Confirm that `documents` is visible as a directory.
11. Only then consider committing Milestone 2B.

---

## 22. TCTBP workflow integration (reconciled 2026-08-12)

This repository has been reconciled with the TCTBP workflow system from the TCTBP-Web template repo. It uses the **staged branch model**: `development` → `staging` → `main`.

### Branch rules

- Active development happens on `development`.
- Never commit directly to `staging` or `main`; promote through the chain.
- `promote staging` merges development into staging; `promote production` merges staging into main; `ship` creates the production release/tag from a clean `main`.
- Short-lived task branches may be created off `development` and closed via `branch <name>`.

### Key files (do not casually edit)

- `.github/TCTBP.json` — machine-readable workflow policy and project profile (source of truth).
- `.github/TCTBP Agent.md` — behavioural rules and guard rails.
- `.github/TCTBP Cheatsheet.md` — operator quick reference.
- `scripts/tctbp-run-*.js` — deterministic workflow runners (plain Node, no dependencies).

### CI (GitHub Actions)

- `.github/workflows/ci.yml` — runs `cargo fmt --check`, `cargo clippy --workspace --all-targets`, `cargo test`, and `cargo build` on pushes to `development`, `staging`, `main`, and `feature/`/`chore/`/`slice/` branches (and via `workflow_dispatch`).
- `.github/workflows/release.yml` — on `v*` tags, verifies the tag matches `[workspace.package] version`, then runs tests and a release build.

### TCTBP triggers handled by the TCTBP agent

`checkpoint`, `publish`, `promote staging`, `promote production`, `ship`, `deploy dev/staging/prod` (not configured yet — desktop distribution pending), `handover`, `handover local`, `resume`, `orient`, `status`, `abort`, `gate test/lint/build`, `rollback`, `version status`.

When these phrases are used, follow the TCTBP agent workflow rather than implementing the steps manually.

### Verification commands (Cargo)

- `gate test` → `cargo test`
- `gate lint` → `cargo clippy --workspace --all-targets`
- `gate build` → `cargo build`
- `gate format` → `cargo fmt --check`
- `format` → `cargo fmt` (apply, not check)

### Versioning

- Single version source: `Cargo.toml` `[workspace.package] version` (the runtime keeps `Cargo.lock` in sync on bump).
- Tags use `v{version}`, created at ship time.

### Handover continuation

`handover` / `handover local` compose a continuation file under `.tctbp/continuation/` so sessions can resume across machines with `orient` or `resume`. Superseded continuation files are moved to `.tctbp/continuation/archive/` (see its README) to keep the directory current.

### Notes

- The reconciled runtime does not include the web-scaffold or hotfix runners; those triggers are intentionally not active.
- Deploy environments are not yet configured (this is a desktop application); `deploy` workflows stop with 'no targets configured'.
- Pre-TCTBP commits were made directly to `main`; that was the pre-workflow convention. From this point, milestone work follows the staged model on `development`.

Keep the scope there unless testing reveals a blocker.
