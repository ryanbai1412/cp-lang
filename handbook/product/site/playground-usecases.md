# Playground use cases

### play/U1: run a program on an input without installing
Need: try an idea in the browser and see its output and time.
Design: editor, input pane, Run (Cmd+Enter); the in-page compiler runs it as `cpl run` would.
Milestone: M1-critical. Status: supported
Evidence: integration `web/tests/smoke.mjs`; differential in `web/scripts/gate.mjs` (browser and CLI agree on 865 programs)

### play/U2: see an error while typing, as the CLI would report it
Need: find out the program is wrong before running it.
Design: live diagnostics with the code, message and help of `cpl check`, linked to the error page.
Milestone: M1-critical. Status: supported
Evidence: integration `web/tests/diagnostics.mjs`

### play/U3: see a lint and apply its fix in place
Need: learn the idiomatic form without leaving the editor.
Design: lint underline, tooltip with the fix, one click applies `cpl lint`'s edits.
Milestone: M1-differentiating. Status: supported
Evidence: integration `web/tests/lint-fixes.mjs`

### play/U4: discover a std function while typing
Need: find the operation you want from what you are holding.
Design: completion after `.` and in names, with signature, named options and cost; parameter hints; inlay types.
Milestone: M1-critical. Status: partial (recall is a ratchet below 100%)
Evidence: integration `web/tests/intelligence.mjs`; `crates/cpl-lsp/tests/completion_replay.rs`

### play/U5: jump from code to its documentation
Need: read the exact contract of a call you are looking at.
Design: Cmd+click (or Cmd+K then Cmd+click for a new tab) opens the std reference entry.
Milestone: M1-critical. Status: supported
Evidence: integration `web/tests/intelligence.mjs`

### play/U6: get the C++ to submit
Need: submit to a judge that only takes C++.
Design: a C++ tab with the one-file output of `cpl build`, copy in one click.
Milestone: M1-critical. Status: supported
Evidence: differential in `web/scripts/gate.mjs` (identical C++ to the CLI)

### play/U7: find why a run trapped or went wrong
Need: see the state that first went wrong.
Design: trap message with local values; Trace tab with the step-by-step state.
Milestone: M1-differentiating. Status: partial (one trap and one trace are tested)
Evidence: integration `web/tests/smoke.mjs` (a trap's message, the Trace viewer opening and replaying an edited call)

### play/U8: find an input where the solution is wrong
Need: a counterexample before rewriting.
Design: Stress tab: solution against a brute force on generated small inputs, the first disagreement shrunk.
Milestone: M1-differentiating. Status: partial (a TLE's first diverging loop isn't named in the browser)
Evidence: integration `web/scripts/stress-parity.mjs` (the browser and `cpl stress` agree on every `@alt` program); `web/tests/smoke.mjs` (the Stress tab shrinks a planted bug)

### play/U9: keep the work across a reload
Need: not lose a half-written solution.
Design: drafts saved per editor until Reset.
Milestone: M1-critical. Status: supported
Evidence: integration `web/tests/drafts.mjs`

### play/U10: share a program with someone
Need: show a teammate or a forum the exact code and input.
Design: Share makes a short link that opens the same code and input; with the share service down it copies the full link, which needs none.
Milestone: M1-differentiating. Status: supported
Evidence: integration `web/tests/smoke.mjs` (a shared link reopens the same code and input); `web/tests/resilience.mjs` (the share API down)

### play/U11: use it on a phone or from the keyboard alone
Need: read and run code on a phone, or without a mouse or sight.
Design: layout fits 390 px; every control has a name and a key; WCAG 2 AA contrast.
Milestone: later. Status: partial (25 known axe-core violations)
Evidence: integration `web/tests/phone.mjs`, `web/tests/a11y.mjs`

### play/U12: start working before the compiler has loaded
Need: type immediately on a slow connection.
Design: the editor is usable at once; checks and Run start when the compiler arrives, with a visible loading state.
Milestone: later. Status: partial (no test of the loading state)
Evidence: integration `web/tests/page-weight.mjs` (bytes before first edit)
