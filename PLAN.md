# Kindling — Product Scope & Milestone 3 Baseline

Status: agreed 2026-08-12 (planning session). Milestones 3–5 are implemented and verified
against the physical Paperwhite (see `.github/copilot-instructions.md` §9). The device
identity/profile slice is also complete — see “Device identity & local library design”.

## Product definition

Kindling is a Rust desktop application (Linux and Windows) for managing books and Kindle
devices, built on the Kindred device-management engine.

The first usable version (v1) is a **read-only device inventory**: connect a Kindle, list
its library as books, and show title, format and size per book. Transfer, removal and a
device-independent local library are later milestones.

## Agreed planning decisions

1. **V1 scope:** read-only device inventory. Transfer/remove and the local library are
   later milestones.
2. **Book identity:** the ASIN embedded in book filenames (`Title_ASIN.kfx`) is the stable
   book key. The title prefix is used for display; filenames may vary but the ASIN is the
   persistent handle.
3. **Book aggregation:** each book is modelled as one `Book` object that groups its main
   content file plus associated sidecars (`.sdr` folder, `.mf`/`.meta`/`.yjf` metadata).
   Raw MTP objects remain accessible underneath.
4. **Metadata timing:** display titles come from filenames. The per-book `.mf`/`.meta`
   files were investigated (Milestone 6) and carry **delivery/caching metadata only** —
   no real titles/authors/covers on this firmware — so filenames remain the title source.
5. **Local library:** a **JSON-based** local library (per-user `library.json` keyed by
   ASIN) for offline management and cross-machine sync — see “Device identity & local
   library design” below. Chosen over a database for v1 (cheap, reversible, shareable).
6. **Windows CI:** a Windows runner is added to CI now to catch cross-platform compile
   issues; physical Windows hardware testing stays deferred.

## Device identity & local library design (resolved 2026-08-12)

Two users, two machines, two Kindles: Ken on Linux, Deb on Windows. Only **one device is
attached at a time** — there is no multi-device selection; `open_first()` stays correct.

- **Device identity:** the stable USB serial labels the attached Kindle. A local JSON
  profile store (`profiles.json`) maps serial → friendly name (“Ken's Kindle”, “Deb's
  Kindle”). Each machine keeps its own store; the JSON file can be exchanged between
  machines so a cross-over plug-in is recognised instead of shown as unknown.
- **Local library:** a per-user JSON library keyed by ASIN supports offline management
  (add/remove/organise without the device) and reconciliation on connect. The device
  remains the source of truth for what is physically on it; the library is the index +
  offline view.
- **Data files:** `profiles.json` (device identity) and `library.json` (book records) are
  local user data, never committed to the public repo; serials are masked in logs/UI.

### Next work slices

1. Profile registry + serial correlation — **done** (`profile.rs`; CLI `identify`,
   `profiles`, `profile add`).
2. Local JSON library (records + reconcile-on-connect) — **done** (`library.rs`; CLI
   `library`, `library reconcile`, `library add`).
3. `.mf`/`.yjf` metadata investigation — **done** (Milestone 6). Conclusion: the per-book
   device files are delivery/caching data (`.mf` JSON delivery metadata, `.meta` download
   cache, `.yjf` whisperstore marker); no titles/authors/covers. Sidecar metadata-handle
   association is now wired into the inventory.
4. Device wiki — **done** (Milestone 7): `docs/device/README.md` is the canonical
   verified-facts reference (storage tree, file-type catalog, transport quirks,
   versioned unknowns, safety notes). Covers found in `system/thumbnails/` (tiny GIFs);
   sideloaded thumbnail hash→book mapping is an open question.
5. Error abstraction — **done**: `KindredError` (`error.rs`) is the unified device-layer
   error over `rusb`/`mtp-rs` with §17 categories (NoDevice, DeviceBusy, Disconnected,
   Timeout, StorageFull, AccessDenied, PermissionDenied, StaleObject, NotFound,
   InvalidObject, UnsupportedModel); unmapped low-level errors preserved as `Usb`/`Mtp`
   for diagnostics. All device-facing functions now return it.
6. GUI framework decision — **done** (2026-08-12): **iced** chosen; see
   “GUI framework decision — RESOLVED” below. Next: the GUI app shell (GUI M1).
7. Kindle Collections — read-only investigation only (see below).

## Target UX — Explorer-style library manager (agreed 2026-08-12)

The Kindling desktop UI follows a file-manager model familiar from Windows Explorer /
Finder:

- **Left pane:** collapsible logical sections — **Local Library** and **Kindle Library**
  (per attached device). These are logical book sections, **not** a raw file tree: each
  row is one aggregated `Book` (never the underlying `.kfx`/`.sdr`/`.mf` objects), so
  storage internals stay hidden and book-group integrity is preserved.
- **Right pane / preview:** metadata panel for the selected book (title, author, format,
  size, ASIN, cover). Note: Milestone 6 showed the per-book device files do not carry
  titles/authors/covers, so the panel shows filename-derived title + device facts; covers
  would need a separate thumbnail-cache investigation.
- **View modes:** covers (default), list, and details/sortable-table — the Explorer
  trio mapped to books.
- **Transfers:** drag-and-drop between panes (local→Kindle = `add_book_to_kindle`,
  Kindle→local = `copy_book_from_kindle`), plus context-menu/button equivalents;
  safe-remove confirmation on any removal (content + sidecar).
- **Status:** per-book badge — on-device / local-only / both — reconciled on connect.
- **Extensibility:** the left-pane section model is data-driven so a future **Collections**
  section can slot in if the investigation pans out.

### GUI framework decision — RESOLVED: iced (2026-08-12)

The UX constrains the framework choice: tree/sections, list + grid + thumbnails, preview
pane, and drag-drop, native-feeling on Windows and Linux. Shortlist evaluated: **Tauri
(+ React)**, **egui/eframe**, **iced**, **Slint**. Decision: **iced**.

Why iced:

- **Declarative/retained architecture (Elm model)** fits a stateful, data-driven library
  manager: `State → View` with a pure `update(message, state)`. Closest Rust analogue to
  the web/React mental model, and the best trajectory for a "friendly" product UI as
  features accrue (selection, preview, view modes, drag state, device presence).
- **Testable by construction:** the pure update function means selection, drag
  transitions and reconciliation are unit-testable without a device or a window —
  matching Kindred's testing conventions.
- **Async is idiomatic:** `Command`/`Subscription` drive Kindred's async MTP futures;
  device hotplug and transfer progress become a `Subscription` pushing messages. iced
  brings the app's async runtime (Tokio); the CLI keeps `block_on` as-is.
- **Retained widgets + `Wrap` and virtualized `Scrollable`** suit cover grids and long
  lists; `iced_table` covers the sortable details view.
- **Pure Rust** — same language as Kindred; no webview, no FFI, no second toolchain.

Why not the alternatives:

- **Tauri (+ React):** strongest web-UI power and packaging, but a two-language stack and
  WebKitGTK variability on Linux. Kept as fallback if the iced spike disappoints.
- **egui/eframe:** superb iteration speed and community, but immediate mode is the wrong
  paradigm for a stateful product app — polish and complex state become manual labour.
  Best reserved for diagnostic/tool UIs (the CLI covers diagnostics).
- **Slint:** ruled out on licensing (GPLv3 free tier) and DSL friction.
- **GTK / Qt:** GTK is alien on Windows; Qt adds binding and licensing friction.

Open items to retire with a spike before full commitment:

- Drag-and-drop between Kindling's own panes is custom work in iced (drag state in the
  Model, rendered drop target, message on drop) — bounded, roughly a day.
- Cover-grid performance with hundreds of covers — verify virtualization in the spike.
- Iteration feel vs egui — the spike settles it with evidence.

App-layer plan:

- New workspace member `apps/kindling-gui` — a thin shell over the `kindred` boundary;
  no device logic in the UI.
- GUI M1: Explorer shell — left section pane (Local Library / Kindle Library) + right
  preview pane + cover grid, mock data first, then real `inventory_device` + `LocalLibrary`.

### GUI M1 — iced Explorer shell — **done** (2026-08-12)

- `apps/kindling-gui` is a workspace member (iced 0.14, `kindred` path dep). Entry via
  `iced::application` (title “Kindling”, 1200×800).
- Small modules per the code-splitting rules: `model.rs` (state/messages/update),
  `mock.rs` (60 invented titles, deterministic ids/statuses), `view/` split into
  `mod.rs` (layout), `sidebar.rs`, `grid.rs` (cover grid via iced `Grid::fluid`),
  `preview.rs`, `theme.rs` (styles + hash-derived pastel cover colours).
- Three-pane Explorer: left sections (Local Library / Kindle Library with counts),
  centre cover grid (status badges, selection highlight), right preview pane, status bar.
- 4 unit tests on the update logic (section switch, selection, section filtering) —
  windowless, matching the testability case for iced.
- Verified: builds, gates green, launches cleanly (no panic) on the dev machine.
- Spike items still open for GUI M2: pane-to-pane drag-and-drop (custom iced work),
  real `inventory_device` + `LocalLibrary` wiring, list/details view modes.
- CI: ubuntu job installs `libxkbcommon-dev libwayland-dev libgtk-3-dev` for iced/winit.

## Kindle Collections — investigation RESOLVED (2026-08-12)

Collections on modern Kindle firmware (12th-gen, 5.19.x) are cloud-synced “Amazon
Collections”, not the older USB-managed local collections. §12 of the instructions flags
this as potentially difficult/unreliable over USB/MTP.

**Outcome of the read-only investigation (2026-08-12):** the full `system/` tree was
walked over MTP on the physical Paperwhite (all subfolders descended; only unrelated DBs
found — search index, device profiles, annotations, FreeTime, fmcache, vocabulary, sync).
No `collections.json`/`collections.db`/`collection.*` exists on the device, so **Amazon
Collections have no MTP-reachable local representation**. Conclusion:

- **Do not** design collection CRUD into the model, UI, or local library schema.
- **Collections are out of scope for the USB transport** unless a Wi-Fi/cloud path is
  later built (separate future transport).
- The GUI left-pane section model remains data-driven, so a future Collections section
  can slot in only if a usable data source appears later.
- Details: `docs/device/README.md` §3 (resolved note) and §5 (versioned unknowns).

## Device evidence base (from Milestones 2B/2C, 2026-08-12)

The full, maintained reference for device structure is **`docs/device/README.md`** — a
verified-facts wiki (firmware-versioned, no identifiers, handles noted as transient).
The summary below is a compact index:

- Storage root contains `documents`, `system`, `fonts`, `screenshots`, `voice`,
  `audible`, plus calibre files (`metadata.calibre`, `driveinfo.calibre`) and
  `FILE_SYSTEM_ACCESSIBILITY_FLAG`.
- Books live in `documents/Downloads/Items01/`: 58 `.kfx` books, each a `Title_ASIN.kfx`
  file paired with a `Title_ASIN.sdr` sidecar folder. The `.yjf`/`.mf`/`.meta` per-book
  metadata files live **inside each `.sdr` sidecar folder** (Milestone 6), not beside the
  content file; they are JSON delivery metadata / a download cache / a whisperstore
  marker — not titles or covers.
- Dictionaries (`.azw`) live in `documents/dictionaries/`.
- `My Clippings.sdr` confirms the `.sdr` sidecar convention at the documents root.
- Send-to-Kindle PDFs are converted to `.kfx` on the device.
- `mtp-rs` returned the complete listing where a libmtp `mtp-files` scan under-reported
  (validates the pure-Rust MTP choice).

## Milestone 3 — Kindle inventory model (next)

Goal: introduce Kindle-aware inventory types above raw MTP object summaries, using the
evidence above.

Scope:

- `Book` model: ASIN key, title (from filename), format, size; groups main file +
  sidecars + metadata objects.
- Recognise supported book formats (`.kfx`, `.azw`; later `.mobi`, `.pdf` etc. as
  evidence appears).
- Locate the book content area (`documents/Downloads/Items01/`) and dictionaries.
- Parse `Title_ASIN` filenames into (title, ASIN).
- Keep raw storage paths/object IDs separate from user-facing book identity.
- Diagnostic CLI command to list the device inventory as books.
- Read-only. No uploads, deletes, moves, or renames (see safety rules §12).

Out of scope for M3 (queued):

- Local library + storage model (own milestone)
- `.mf`/`.meta` metadata parsing (own milestone)
- Transfer/copy and removal (Milestones 4/5)
- Error abstraction (Kindred error enum over `mtp_rs::Error`)
- Device selection / USB↔MTP correlation for multiple Kindles
- GUI framework selection + async runtime decision — **resolved** (2026-08-12: iced; see
  the GUI decision section above)

Definition of done for M3:

- `cargo run -p kindling-cli -- inventory` lists the physical device's books (title,
  format, size, ASIN) grouped correctly with their sidecars.
- Gates pass (fmt/check/test/clippy); TCTBP checkpoint + publish on `development`.
- Verified against the physical Paperwhite; output recorded in the instructions.
