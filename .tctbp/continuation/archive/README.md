# Handover Archive

Old handover continuation files are moved here so `.tctbp/continuation/` stays current.

## Convention

- `handover` / `handover local` write a timestamped file `YYYY-MM-DD-HHmm.md` into
  `.tctbp/continuation/`.
- When a continuation file has been superseded (a newer one was picked up with
  `orient` or `resume`), move the old file into this archive instead of deleting it.
- Optional: group archived files by month, e.g. `archive/2026-08/`.

`orient` and `resume` read the newest continuation file from `.tctbp/continuation/`
(the archive is for record-keeping only).
