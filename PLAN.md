# Kindling — Product Scope & Milestone 3 Baseline

Status: agreed 2026-08-12 (planning session). See `.github/copilot-instructions.md` for
the project context, safety rules and device evidence.

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
4. **Metadata timing:** display titles come from filenames in M3. Parsing the per-book
   `.mf`/`.meta` metadata files (real titles/authors/covers) is a later milestone.
5. **Local library:** introduced after on-device inventory, as its own milestone (implies
   a storage decision: files vs database).
6. **Windows CI:** a Windows runner is added to CI now to catch cross-platform compile
   issues; physical Windows hardware testing stays deferred.

## Device evidence base (from Milestones 2B/2C, 2026-08-12)

- Storage root contains `documents`, `system`, `fonts`, `screenshots`, `voice`,
  `audible`, plus calibre files (`metadata.calibre`, `driveinfo.calibre`) and
  `FILE_SYSTEM_ACCESSIBILITY_FLAG`.
- Books live in `documents/Downloads/Items01/`: 58 `.kfx` books, each a `Title_ASIN.kfx`
  file paired with a `Title_ASIN.sdr` sidecar folder, plus `.yjf`/`.mf`/`.meta` per-book
  metadata.
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
- GUI framework selection + async runtime decision

Definition of done for M3:

- `cargo run -p kindling-cli -- inventory` lists the physical device's books (title,
  format, size, ASIN) grouped correctly with their sidecars.
- Gates pass (fmt/check/test/clippy); TCTBP checkpoint + publish on `development`.
- Verified against the physical Paperwhite; output recorded in the instructions.
