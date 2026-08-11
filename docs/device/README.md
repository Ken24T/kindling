# Kindling — Paperwhite Device Wiki

Reference knowledge about the physical Kindle hardware Kindling targets.
This is the **canonical home for device facts**; the project instructions
(`.github/copilot-instructions.md`) hold rules, milestone narratives and pointers,
and `PLAN.md` holds decisions. When writing code that touches the device, read this
file first.

## 0. How to read this wiki

- Every claim is marked with the firmware it was verified on: `[5.19.5]` = verified
  against **Kindle Paperwhite 12th gen, firmware 5.19.5 (build 479431058)**.
- `[verified]` = observed directly on the physical device. `[unverified]` = inferred
  or reported, not yet confirmed on hardware.
- **MTP handles are transient.** They change every session/reconnect. Nothing in this
  wiki records handles as a way to find objects — navigate by name, or list fresh at
  runtime. (Observed in this very session: a second reconnect produced different
  handle numbers for the same folders.)
- **No identifiers or personal data.** No USB serials, no device UUIDs, no per-user
  identifiers, no personal library inventory. Sample filenames appear only where they
  are already established in the project records and are structural examples.
- The wiki **describes** the device. Kindred code must keep **discovering adaptively at
  runtime** — never treat this wiki as a spec that code reads to decide device layout.
- USB bus/device addresses are transient and must never be used as identity.

---

## 1. Device header

| Property | Value | Status |
| --- | --- | --- |
| Model | Kindle Paperwhite, 12th generation | [verified] |
| Firmware | 5.19.5 (build 479431058) | [verified] — read from `system/version.txt` |
| USB vendor ID | `0x1949` (Amazon) | [verified] |
| USB product ID | `0x9981` | [verified] |
| USB manufacturer | `Amazon` | [verified] |
| USB product string | `Kindle Paperwhite` | [verified] |
| USB serial | present and **stable across replug** — use as local identity key | [verified] |
| MTP manufacturer / model | `Amazon` / `Kindle Paperwhite` | [verified] |
| MTP storage | `Internal Storage` — generic hierarchical, read/write | [verified] |
| Storage capacity / free | ~12.55 GB / ~11.85 GB (probe) | [verified] |
| Network capability | Wi-Fi | [verified] |
| Bluetooth | present on hardware but exposed for audio/accessibility only — **not** a data transport | [verified] |

Identity rules (from project instructions §3): store real serials only in local
config/database, mask in logs/UI, never commit. `MTP device-info` on this model does
**not** expose the serial — identity comes from USB discovery, not MTP.

---

## 2. Transport facts

### USB discovery (Linux)

- `lsusb` shows `1949:9981 Lab126, Inc. Kindle Paperwhite`.
- Kindred reproduces discovery in pure Rust via `rusb` (`usb.rs`): filters Amazon VID,
  recognises PID `0x9981`, reads descriptors, and still reports discovery when optional
  string descriptors fail.
- The **bus number / device address change across reconnects** (observed). Never identity.

### MTP

- The Paperwhite presents user storage via **MTP**, not a mounted mass-storage volume.
- Kindred talks MTP via the pure-Rust **`mtp-rs`** crate — no `libmtp` dependency.
- `mtp-rs` successfully: opens the device, reads model/storage, enumerates folders and
  files, uploads, downloads, deletes, and creates folders on the physical device.
- **`mtp-rs` vs libmtp divergence (important):** `mtp-rs` returns the *complete* folder
  listing where libmtp's `mtp-files` recursive scan under-reported and **misattributed
  parents** (the `.mf`/`.yjf`/`.meta` metadata files were reported by libmtp under
  `Items01/` but actually live inside each `.sdr` sidecar folder). Trust `mtp-rs`
  listings; treat libmtp output as suspect.
- libmtp's static device list does not know `1949:9981` but still communicates fine once
  the interface is free. Not a blocker.

### Linux ownership conflict (GVFS)

- Desktop MTP services (`/usr/libexec/gvfsd-mtp`) may claim the MTP interface and block
  other clients with a busy/claim error. Opening the Kindle in a file manager can
  trigger this.
- Diagnosis: `ps -ef | grep -Ei '[g]vfsd-mtp|[k]iod[56]|[k]io.*mtp'`.
- Development workaround (not a product solution): `pkill -x gvfsd-mtp`.
- Kindling must ultimately *detect and report* "device in use by another client" with a
  friendly message — not a low-level panic — and avoid requiring root.

---

## 3. Storage tree map

Layout of `Internal Storage` (root) — `[verified, 5.19.5]` unless noted. Navigate by
name at runtime; handle numbers are session-local.

```text
/ (storage root)
├── documents/                     ← user content area (Kindling's focus)
│   ├── .cache/kf8/                [verified] per-book MD5-named cache files (+ .settings)
│   ├── My Clippings.sdr/          [verified] clippings sidecar at documents root
│   ├── dictionaries/              [verified] .azw dictionaries
│   ├── Downloads/Items01/         [verified] main book area: Title_ASIN.kfx + .sdr pairs
│   └── <book>.kfx                 [verified] root-level sideloads (e.g. restored books)
├── screenshots/                   [verified] empty at capture
├── voice/                         [verified] TTS voice-pack dirs: english, spanish,
│                                              french, german, italian, portuguese
├── system/                        [verified] firmware internals — see below
├── audible/                       [verified] default.hushpuppy.db, hushpuppy-internationalization.xml
├── fonts/                         [verified] user font area; Readme.txt (sideload instructions)
├── FILE_SYSTEM_ACCESSIBILITY_FLAG [verified] 16-byte file, blank content; semantics unknown
├── driveinfo.calibre              [verified] calibre JSON state — do NOT depend on
└── metadata.calibre               [verified] calibre JSON book metadata — do NOT depend on
```

### `system/` (firmware internals)

`[verified, 5.19.5]` top-level contents:

```text
system/
├── version.txt                    ← "Kindle 5.19.5 (479431058)" — firmware source of truth
├── thumbnails/                    ← cover/thumbnail cache — see File type catalog
├── bookcovers/                    [verified] one hash-named subdir; contents not descended
├── grok_thumbnails/               [verified] empty at capture (grok/AI feature cache?)
├── readingstreams/, wmtlogs/, btlogs/, userannotlogsDir/   ← log areas
├── freetime/, startactions/, recommendation/               ← firmware feature areas
├── Search Indexes/, CloudIndices/, .fastSync_v1/, .fastsync_cache/, fmcache/, cmm/, kll/, ksdk/, vocabulary/, pdf/, preloader/, acw/
├── AudibleJit.sys, AudibleActivation.sys                   ← audible activation state
└── fonts/
```

### `documents/` detail

- `documents/Downloads/Items01/` holds the main book content: `Title_ASIN.kfx` files
  paired with `Title_ASIN.sdr` sidecar folders (58 books at capture). Also contains
  `.pdf`-sourced sideloads converted to `.kfx` (e.g. `Kindle_Rustic_Pan_Lamb_Curry.pdf_<id>.sdr`).
- `documents/dictionaries/` holds `.azw` dictionaries.
- `documents/` root also receives root-level sideloaded books (a book restored via
  `add-book` lives there, not in `Items01/`).
- Books are **not** assumed to sit at the `documents/` root — the inventory walks the
  whole tree.

---

## 4. File type catalog

Per-book and device file types observed on the device. `[verified, 5.19.5]`.

| Extension / name | Where | What it is | Usable metadata? |
| --- | --- | --- | --- |
| `.kfx` | `Items01/`, `documents/` root | Book content (`Title_ASIN.kfx`). KFX (KF8-derived) format | title/ASIN from filename convention |
| `.azw` | `dictionaries/` | Dictionary content | title from filename |
| `.sdr/` (folder) | beside each `.kfx` | Per-book sidecar folder | — |
| `.sdr/<ASIN>.mf` | inside `.sdr/` | JSON **delivery metadata** (content id, `kindle.pdoc` type, Amazon delivery endpoints) | ❌ no title/author |
| `.sdr/AssetDownloadMetadata.meta` | inside `.sdr/` | JSON download cache (ETags, Last-Modified, Last-Downloaded) | ❌ no title/author |
| `.sdr/<Title_ASIN><hash>.yjf` | inside `.sdr/` | 113-byte binary `whisperstore.migration.status` marker | ❌ no title/author |
| `.sdr/assets/` | inside `.sdr/` | empty on tested book — covers are **not** per-book here | ❌ |
| `.sdr/data/` | inside `.sdr/` | pagination cache (`.pagination.cache/`) | ❌ |
| `version.txt` | `system/` | firmware version string | — |
| `thumbnail_<ASIN>_EBOK_portrait.jpg` | `system/thumbnails/` | store-book cover thumbnail — **actually GIF89a, 60×40, despite `.jpg` extension** | cover pixels, tiny |
| `thumbnail_<hash>.jpg` | `system/thumbnails/` | sideloaded-book cover thumbnail (opaque 6-char hash name) | cover pixels, tiny; hash→book mapping unknown |
| `FILE_SYSTEM_ACCESSIBILITY_FLAG` | root | 16 bytes, blank content | semantics unknown |
| `driveinfo.calibre` | root | JSON calibre state (device_store_uuid, device_name, location_code, last_library_uuid, calibre_version, date_last_connected, mtp_prefix) | calibre-private; **do not depend** |
| `metadata.calibre` | root | JSON array of book records (title, authors, languages, size, lpath…) for calibre-managed books only | real titles/authors **but only for calibre-managed books**; **do not depend** (§12) |
| `default.hushpuppy.db`, `hushpuppy-internationalization.xml` | `audible/` | audible state DB + i18n | — |
| `.cache/kf8/<md5>` (+ `.settings`) | `documents/.cache/kf8/` | per-book MD5-named cache files (reading/font caches) | — |

### Cover availability (important for the UI roadmap)

Covers **do** exist on the device, in `system/thumbnails/` — not per-book in sidecars.
Two naming schemes:

- `thumbnail_<ASIN>_EBOK_portrait.jpg` — store books keyed by **classic 10-char ASIN**
  (e.g. `B00KAJJRIM`). Note these are classic ASINs; the 58 `Items01/` books mostly use
  32-char hex ids, so the thumbnail cache may cover a different (store) set.
- `thumbnail_<hash>.jpg` — sideloaded books keyed by an opaque 6-char hash; the
  hash→book mapping is **unknown** (open question).

Thumbnails are tiny (60×40) GIFs in `.jpg` clothing — fine as e-ink previews, not as
high-res covers.

---

## 5. Versioned unknowns / open questions

`[unverified]` — do not build on these until device evidence resolves them.

- **`FILE_SYSTEM_ACCESSIBILITY_FLAG`** — present, 16 bytes, blank. Semantics unknown.
  Do not treat as a book.
- **Kindle Collections** — modern firmware uses cloud-synced "Amazon Collections", not
  USB-managed local collections. No on-device collection metadata found yet; likely not
  manageable over MTP. Read-only investigation candidate: `system/`.
- **Sideloaded thumbnail hash** — how `thumbnail_<hash>.jpg` maps to a book is unknown;
  a future investigation could correlate with ASIN/title hashes to enrich the library
  with covers.
- **`bookcovers/<28-hex>/`** — one hash-named subdir observed; contents not descended.
- **`grok_thumbnails/`** — empty at capture; purpose (grok/AI features) unconfirmed.
- **`system/` subfolders** (`readingstreams`, `freetime`, `recommendation`,
  `CloudIndices`, `Search Indexes`, `kll`, `ksdk`, `cmm`, `acw`, `preloader`, …) —
  names observed; semantics not investigated. Treat as firmware internals, do not modify.
- **`.cache/kf8/`** — MD5-named files; purpose inferred as reading/font caches.
- **`voice/`** — TTS voice-pack directories; contents not descended.
- **Firmware updates** — all `[5.19.5]` facts are suspect after a firmware update;
  re-verify before relying on them.

---

## 6. Safety & interop notes

From project instructions §12:

- **Read-only by default.** Until a write milestone is explicitly approved, no uploads,
  deletes, moves, or renames.
- **Never modify `system/`** — firmware internals. `system/version.txt` and the cover
  caches are read-only references.
- **Do not depend on calibre's private files** (`metadata.calibre`, `driveinfo.calibre`)
  unless a separately justified interoperability requirement appears. They may be
  rewritten or absent; `metadata.calibre` only covers calibre-managed books.
- **Do not modify calibre files merely because they exist.**
- **Kindle Collections** must not be assumed manageable via ordinary file operations.
- `FILE_SYSTEM_ACCESSIBILITY_FLAG` and other root markers are device semantics, not books.
- No jailbreak-specific behaviour.
- Calibre has been used with this device (`calibre_version` 9.11.0 observed in
  `driveinfo.calibre`); that is environment context, not an application dependency.

---

## 7. Provenance

All `[verified]` facts above were captured on **2026-08-12** against a physical
Kindle Paperwhite 12th gen, firmware **5.19.5 (479431058)**, via Kindred's `mtp-root`,
`mtp-documents`, `mtp-folder` commands and `mtp-getfile`/`file`/`strings` inspection.
See `.github/copilot-instructions.md` §9 (Milestones 2A–6) for the milestone-by-milestone
evidence trail and commit references.
