# debugger

Segment: E. Experience. Owns: `crates/cpl-stress`, `cpl-trace` (split out), `web/packages/trace-viewer`. Worklog: [worklog.md](worklog.md). Architecture: [ARCHITECTURE.md](ARCHITECTURE.md).

## Covers
`cpl stress` (generators, shrinking, interactors, wasm; merged from stress), interpreter UX (traps with values and call path), `cpl run --trace`, the trace viewer, the debugger.

## Improve
Split into `crates/cpl-trace` + `web/packages/trace-viewer`; trace overhead < 3x; browser test for the viewer; re-capture screenshots on v0.3; DAP later; stress: shapes a row can't describe, graph/tree/grid/anti-hash generators, shrinking in the browser.

## Success criteria
The stress bug benchmark (21 bugs, grown to the P0 classes) finds and minimizes every bug, median < 1 s, also in the playground; tracing costs < 3x plain interpretation; every trap shows source-level values; viewer browser test in the gate; playground Trace works on every runnable docs example.

## Ryan's role
Aligns on the workflow (WA -> stress -> trap -> trace -> viewer) and what generators must describe, then reviews rounds on recordings.
