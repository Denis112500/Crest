# Crest — standing rules

A small personal Tauri 2 desktop app: a pill at the top center of the screen (like the iPhone Dynamic Island) that shows what's playing. Owner is a CS student new to Rust, Tauri and Windows: explain what and why, give exact commands, keep steps small, and say explicitly when something is worth writing in their own notes (point to the matching notes entry).

## Every session
1. Read `STATE.md` first: where the code is, what works, what's in progress, the next task. Do not read `notes/` unless the task needs it; the index below says which file covers what.
2. If this file conflicts with the user's request, say so instead of choosing silently.
3. Tools for Claude's own work (Claude Code plugins, skills, dev-tools helpers) may be installed without asking; say what and why afterwards. Anything that ships inside Crest (Rust crates, npm packages) or installs a program system-wide is named first, in one line: what, why, size.
4. Work milestone by milestone. Each milestone must be runnable. After each one: how to test it, what should be learned, which files changed and what each does, and when to take notes.

## Notes (`STATE.md` and `notes/`)
- Local only: both are git-ignored, never published. Their backup is a private GitHub repository (`crest-notes`); on "commit notes", copy `STATE.md` and `notes/` there, commit ("Update notes") and push.
- `STATE.md`, about 50 lines: where the code is, what works, what's in progress, what's broken, the next task. A new session must be able to start from it alone.
- `notes/<topic>.md` is the project log, one file per topic. Entries newest first: date, milestone/topic, then Done / Learned / Decisions / Problems / Open questions / Next. An entry goes into the topic it is mostly about.
- Update the matching notes file after every milestone and every significant decision, problem or discovery, without being asked. A new standing rule also gets a line in `notes/rules-learned.md`.
- Mark research findings as verified by testing, verified from docs, or unverified.
- Short and concrete. No filler, no repeating code. Reusable commands go in `notes/commands.md`.
- Neutral voice: "Decision:", "Verified by testing:", "Problem:", without "(user)", "(mine)", "I", "my" or "the user" for the people working on the project. Name a place or method instead when it matters ("by hand outside the Claude app"). Never attribute work to a person who didn't do it.
- Never delete old entries; add corrections under them.
- End of session: update `STATE.md` with what changed and the next task.

### Notes index (read a file only when the task needs it)
- `notes/rules-learned.md`: before testing, measuring, committing or releasing.
- `notes/claude-code-integration.md`: anything Claude Code (hooks, pipe, probe, sessions, permission requests).
- `notes/pill-window-and-layout.md`: the pill's window, shape, morph, layout, visibility.
- `notes/fullscreen-detection.md`: the pill hides or doesn't appear unexpectedly.
- `notes/music-smtc.md`: the music activity, players, album art, buttons.
- `notes/settings-window.md`: the settings window, launch at login, players, display choice.
- `notes/release-and-install.md`: before a release, an installer or autostart work.
- `notes/measurements.md`: before measuring memory or CPU.
- `notes/app-architecture.md`: changing the core (arbiter, sources, IPC).
- `notes/project-process.md`: planning phases, rules, publishing, launch, look pass (with the inspiration list).
- `notes/commands.md`: reusable commands. `notes/ideas.md`: app ideas not planned yet.
- `notes/archive/`: never by default (finished history, and the original `notes.md` of 2026-10-09).

## Scope and platforms
- Primary: Windows 11. Later: Linux (Arch, KDE Plasma, Wayland) — not built yet, but all platform-specific code (media backend, window behavior) lives behind small traits in its own folder.
- Scope follows the roadmap (`STATE.md`; history in `notes/project-process.md`). Not planned yet: Linux support (Phase 5), notifications, volume/brightness popups. New activity sources (integrations) are added one at a time, each approved before it's built.
- Local by default: no network calls unless the user has switched on an integration that needs them, and then only to that integration's own service. Every integration is off by default.
- Free APIs only, never paid ones (no paid plans, no free tiers that require a payment method). Say before building whether a free API still needs a key or token.
- Keys and tokens are stored only in Windows Credential Manager (behind a small trait, like the other platform code), never in files, settings.json, logs, the repo or the page. The page may hand a token to Rust once to store it, but never reads it back.
- Network calls come from Rust; the page's CSP keeps blocking the network.
- No telemetry, analytics or remote crash reports.
- Music data only through Windows SMTC; no unofficial YouTube Music APIs, scraping or cookies.
- Keep idle CPU and memory low: event-driven, no polling loops. One exception: an integration that can only fetch (weather, calendar) may use a coarse timer, and only while it's switched on.

## Reference project
`../reference/coucou` (MIT code; name, Mochi character, icon, sounds and media are rights reserved) is read-only inspiration for window and event patterns. Don't copy code; if a substantial piece would be reused, stop and ask first. Never copy or imitate its name or assets. Verify its patterns against the Tauri docs.

## Code organization (strict)
- One responsibility per file; name the file after what it does. No "utils"/"helpers".
- Split files past ~150 lines or when concerns mix, and say why. Don't over-split (no re-export-only files, no 10-line files for their own sake).
- Distinctive, self-describing names for everything; long and clear beats short. No single-letter names (except trivial loop indexes), no vague names (`data`, `info`, `handle`, `manager`, `temp`, `process`).
- No duplicated names across the codebase: each name points to exactly one thing.
- Comments explain WHY, not what.
- No dead code, commented-out code or leftover debug prints.
- Magic numbers (delays, sizes, colors) go in one constants file per layer: `src-tauri/src/backend_constants.rs`, `src/frontendConstants.ts`, `src/styles/designTokens.css`.
- Rust: one module per concern, thin `mod.rs` files that only declare and re-export, platform code in clearly named folders.
- TypeScript: one exported component or function per file, named after the file. No UI framework.

## Commands
```powershell
npm run tauri dev          # run the app with hot reload
cd src-tauri; cargo build  # compile the Rust side only
cd src-tauri; cargo test   # Rust unit tests
npx tsc --noEmit           # type-check the frontend
```
