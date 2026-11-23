# Ryan's feedback → rules and checks

Each piece of Ryan's feedback becomes a rule, and a mechanical check where one is possible, so the coordinator and the lanes catch it next time without him. Hourly critic pass: sample about 5 reader-facing surfaces and review them against this list.

| Feedback | Rule | Check | Owner |
|---|---|---|---|
| Grid DP doesn't use `Cell`; `r.prev` instead of `p.up` | Grid code uses `type Cell = (Row, Col)`, `p.up`/`p.left` | lint: manual `r - 1`/`c - 1` neighbours | stdlib, idioms |
| Interval DP doesn't use `V.ranges` | Specced features get implemented before ports work around them | spec-vs-impl gap list | stdlib, spec |
| O(1) `step.sum` proposed as a special case | No special cases: cost belongs to the receiver type's method; no std privilege | inventory of codegen intrinsics; no new intrinsic without an entry | compiler, tech-debt |
| Always look for the most general fix | Make it, or record it in the domain PLAN.md under "General fixes owed" | — | all |
| Std doc examples unreadable | Multi-line examples with shared setup | example > 80 chars or containing `;` fails | stdlib, site |
| `O(n)` as plain text | LaTeX at the source (`\(O(n)\)`), KaTeX on the site, Unicode fallback | invalid math on a cost line fails | stdlib, spec, site |
| `up` in prose not linked | `[`name`]` intra-doc links | an unresolved link fails | site, stdlib |
| Couldn't understand `reroot` | Docs standard: purpose, parameter roles, a recognisable example, cost | per-item writable-from-docs test | stdlib, idioms |
| A port in a shared sample didn't compile | The corpus compiles on every land | `cpl check` over all ports | infra |
| Opaque helper names (`net`, `twice`) | Names say what they mean (low priority) | — | idioms |
