# Archived: v0.2 CLI test programs

122 programs (`.cp` with `.in`/`.out`) from `crates/cpl-cli/tests/programs/`
that are written in v0.2 and fail `cpl check` under v0.3. Main only passed
them because the gate forced v0.2. They were archived, not ported, when
v0.2 was deleted (2026-11-22).

Queued: classify each failure against the spec. A valid v0.3 program that
fails becomes a minimal failing case in `spec/conformance/` with its rule;
old v0.2 API gets no case. 5 of them pass `cpl check` but fail the
interpreter vs C++ differential test.
