# PULQVA Kernel Contract v1

This document defines the product identity. Implementations may change; these outcomes do not.

## Product contract

PULQVA is a privacy-first desktop intent-to-file application.

The default user journey is:

**download package -> unpack/open -> type a natural-language request -> receive real choices -> click download (or use Autopilot) -> receive the file**

## Kernel invariants

- **ONE PACKAGE** — release is delivered as a self-contained user package per supported OS.
- **ZERO CONFIG** — default path requires no manual dependency installation or configuration.
- **NATURAL LANGUAGE FIRST** — a URL is optional, not required for the primary workflow.
- **RESULT CHOICE** — the user can see candidate results before downloading.
- **AUTOPILOT** — an explicit mode may automatically choose the best matching result.
- **ONE-CLICK RETRIEVAL** — after selection, no unnecessary configuration questions.
- **NO ACCOUNT REQUIRED** — default workflow requires no service account.
- **NO API KEY REQUIRED** — default workflow requires no user-provided API key.
- **FREE/ANONYMOUS INTENT PATH** — the default intent-understanding path must not identify the user to an AI provider by an account/API key.
- **TOR BY DEFAULT** — external network operations use Tor in the default privacy path.
- **FAIL CLOSED** — failure of the privacy transport must stop external network operations rather than silently fall back to direct networking.
- **WINDOWS + LINUX** — both are first-class release targets once v1 is declared.

## Not frozen by the kernel

The following are replaceable implementation details:

- React / Solid / other UI framework;
- Vite / Rolldown / Farm / Rsbuild / other build tooling;
- Tauri or another compatible desktop shell;
- the specific anonymous AI provider;
- yt-dlp or future retrieval backends;
- ranking algorithm;
- UI visual design;
- internal storage format.

Those components may evolve as long as the kernel contract remains true.
