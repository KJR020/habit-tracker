# Work Tracker (macOS) - Requirements.md

## 1. Overview

A personal, local-first work tracking tool for macOS.

The tool runs in the background, periodically captures screenshots, records active application/window metadata, and stores logs in SQLite. Later phases add OCR + LLM-based activity classification and daily feedback generation.

Primary goal: **reduce manual tracking to near-zero**, then enable **end-of-day review** and actionable feedback.

---

## 2. Goals

### MVP Goals (must-have)
- Periodically capture a screenshot on macOS.
- Record the active application name and (if possible) window title.
- Store metadata in SQLite.
- Store images on disk (DB stores file paths).
- Provide a daily report: timeline + time-by-app summary.

### v1 Goals (next)
- OCR screenshots and store extracted text.
- Rule-based + LLM-based activity classification (with confidence).
- Allow manual correction of classifications (optional UI; CLI-based edits acceptable).

### v2 Goals (later)
- Generate end-of-day feedback (strengths, context switching, suggestions).
- Goals/tasks linkage for feedback (optional).
- Privacy enhancements (exclude rules, masking, encryption, retention policies).

---

## 3. Non-Goals (for MVP)
- Perfect activity classification accuracy.
- Storing full image blobs inside SQLite.
- Continuous video recording.
- Cloud sync / multi-device.

---

## 4. Functional Requirements

### 4.1 Capture Loop (MVP)
- The tracker runs continuously until stopped.
- Configurable capture interval (default: **60 seconds**).
- Each cycle:
  1. Collect active app name (required).
  2. Collect window title if available (optional; empty string allowed).
  3. Capture screenshot to disk (JPEG).
  4. Insert a row into SQLite referencing the screenshot path and metadata.

#### Screenshot capture
- Use macOS builtin command:
  - `screencapture -x -t jpg -q <quality> <path>`
- Defaults:
  - `-t jpg`
  - `quality = 60` (configurable)

#### Pause / Resume (MVP "emergency brake")
- Support pausing screenshot capture without stopping the daemon.
- Implementation option (acceptable):
  - If a file `~/.work-tracker/pause` exists, do not capture screenshots.
- When paused:
  - Option A: store a DB row with `is_paused=1` and no image path
  - Option B: store nothing (acceptable for MVP)
- Provide CLI commands:
  - `tracker pause`
  - `tracker resume`

### 4.2 Metadata Collection (MVP)
- Active app name:
  - Obtained via `osascript` / AppleScript.
- Window title (best-effort):
  - Obtained via `osascript` / AppleScript; may fail for some apps.
- Record:
  - `captured_at` (ISO-8601 local time)
  - `active_app`
  - `window_title`
  - `image_path` (absolute or relative path)

### 4.3 Storage (MVP)
- SQLite database file:
  - default: `~/.work-tracker/tracker.db`
- Images directory:
  - default: `~/.work-tracker/images/YYYY-MM-DD/HHMMSS.jpg`

#### Schema (MVP)
- `captures` table:
  - `id INTEGER PRIMARY KEY AUTOINCREMENT`
  - `captured_at TEXT NOT NULL`  (ISO-8601)
  - `image_path TEXT`            (nullable when paused)
  - `active_app TEXT NOT NULL`
  - `window_title TEXT NOT NULL DEFAULT ''`
  - `is_paused INTEGER NOT NULL DEFAULT 0`
  - `is_private INTEGER NOT NULL DEFAULT 0` (reserved for later)
- Index:
  - `INDEX(captured_at)`

### 4.4 Reporting (MVP)
Provide CLI reports:
- `tracker report --date YYYY-MM-DD`
- `tracker report --today`

Outputs (plaintext or markdown acceptable):
1. **Timeline**
   - rows: time, app, title, (image path optional)
2. **Time by App**
   - Estimate duration by counting consecutive captures:
     - duration per capture = interval seconds
   - Aggregate by `active_app`

Optional (nice-to-have):
- Export JSON/CSV:
  - `tracker export --date ... --format json|csv`

---

## 5. Future Requirements (Post-MVP)

### 5.1 OCR (v1)
- For each capture, extract text from screenshot:
  - Store in table `ocr_results`:
    - `capture_id`
    - `text`
    - `engine`
    - `confidence` (optional)
- MVP may skip OCR entirely.

### 5.2 Classification (v1)
- Produce an activity label per capture (e.g., Coding / Slack / Docs / Meeting / Browsing).
- Store in table `classifications`:
  - `capture_id`
  - `label`
  - `confidence`
  - `method` (rule|llm)
  - `model` and `prompt_version` (for LLM)
- CLI command:
  - `tracker classify --date YYYY-MM-DD` (batch mode)

### 5.3 Daily Feedback (v2)
- Summarize the day and generate feedback text.
- Inputs:
  - App/time aggregation
  - Classification timeline
  - OCR snippets (optional)
- Output:
  - Markdown report saved to `~/.work-tracker/reports/YYYY-MM-DD.md`

### 5.4 Privacy + Retention (v2)
- Exclude rules by app name / title regex.
- Optional masking.
- Retention policy:
  - Delete screenshots older than N days (configurable)
  - Keep metadata longer

---

## 6. Technical Stack

### 6.1 Language / Runtime
- Rust (single binary CLI + background mode).

### 6.2 macOS Integration (MVP)
- AppleScript via `osascript` for:
  - active app name
  - window title (best-effort)
- Screenshot via `screencapture`.

### 6.3 DB
- SQLite
- Rust crate: `rusqlite`

### 6.4 Time
- Rust crate: `chrono`

### 6.5 CLI
- Rust crate: `clap`

### 6.6 Logging (optional)
- `tracing` + `tracing-subscriber`

---

## 7. Configuration

### Config file (recommended)
- `~/.work-tracker/config.toml`

Example:
```toml
interval_seconds = 60
jpeg_quality = 60
db_path = "~/.work-tracker/tracker.db"
images_dir = "~/.work-tracker/images"
pause_file = "~/.work-tracker/pause"


⸻

8. Permissions / Setup Notes (macOS)
	•	Screen Recording permission is required for screenshots.
	•	Accessibility permission may be required for querying window titles through System Events.

For MVP:
	•	If window title retrieval fails, it must not crash the tracker; store empty title.

⸻

9. CLI Commands (MVP)
	•	tracker start [--interval N] [--quality Q]
	•	Starts the capture loop in foreground.
	•	tracker pause
	•	Create pause flag.
	•	tracker resume
	•	Remove pause flag.
	•	tracker report --date YYYY-MM-DD
	•	Print timeline + time-by-app.
	•	tracker report --today
	•	Shortcut for today.

Nice-to-have:
	•	tracker export --date YYYY-MM-DD --format json|csv

⸻

10. Acceptance Criteria (MVP)
	•	Running tracker start captures screenshots at the configured interval.
	•	Each capture inserts a row into SQLite with correct timestamps and app metadata.
	•	Images are written to the expected directory structure.
	•	tracker pause stops new screenshots from being saved (without terminating the process).
	•	tracker report --today prints a usable daily summary.

