# Decision queue

Calls only Ryan makes: ones that change the language's character, and the design reviews for segment C. Everything else is an ordinary decision the owner makes and logs in `decisions.md`.

One entry per call, newest at the bottom of "Open":

```
### <domain>: <question>
Options, each with real code from a port or spec case.
Recommendation and why.
How to reverse it.
Meanwhile: the reversible default the lane works with.
```

Lanes never block on an entry: they work on the default and on everything else. When Ryan answers, the entry moves to "Answered" with the answer, and the decision goes into `decisions.md`.

## Open

### diagnostics: placeholder fix-its, or only right ones?
A broken line like `let (n, k) =` (aac4p6 `spec.cp`, recovery edit "cut the value after `=`") today gives `error[E0102]: expected a value after `=`, found the end of the line` with no fix-it.
Option A (today): fix-its only where the repair is the right one (`a && b` -> `a and b`, `!x` -> `not x`, a misspelled field -> its near name). Broken-ports fix-it rate stays 72.4%; the "cut the value" row stays at 0%.
Option B: also offer placeholders, e.g. `let (n, k) = todo`, so the program checks and runs to the hole. The rate would rise, but a fix that compiles and is wrong is the contestant's next bug.
Recommendation: A. A fix-it is a promise that applying it is correct; placeholders spend that trust, and incomplete programs already run.
How to reverse it: add placeholder fixes at the E0102 site and count them in the scoreboard; no other code depends on their absence.
Meanwhile: A; the "cut the value" row is excluded from the fix-it target.

### tech-debt: which names may the compiler know (the builtin floor)?
Today `crates/cpl-ir/src/lower/call.rs` matches 89 names, from `print` and `popcount` to library shapes like `prefix`, `first_ge`, `adjacency`, `laws_hold`; a user library cannot do the same.
Options: (a) floor = I/O (`read`, `print`, `flush`), numeric primitives (`abs`, `min`, `max`, bit ops, `sqrt`), storage of `List`/`Str`/`Map`/`Set`; each declared in `std/` with an `@intrinsic` marker any library may use; everything else written in CP Lang. (b) keep the current set, freeze it. (c) no floor, only "goes down".
Recommendation: (a), about 30 names; it is what "no privileged stdlib" means and makes every remaining special case visible in `std/`.
How to reverse it: a name moves back from `std/` to the floor list with a `decisions.md` line.
Meanwhile: `ir_builtin_names` may only go down; no new name is added without a `decisions.md` line.

### tech-debt: may the steward land pure-move refactors in other lanes' crates?
Example: splitting `crates/cpl-interp/src/vm.rs` (15,367 lines) into modules with no behaviour change.
Options: (a) yes, behaviour-free moves only, announced first in the owner's worklog; (b) proposals only, owner does every move.
Recommendation: (a); moves are mechanical, conflict-prone and cheapest when one lane batches them.
How to reverse it: owners take moves back; the steward files proposals only.
Meanwhile: (a) for `vm.rs` (agreed in the previous round), proposals for every other file.

### tech-debt: how strict are the debt ratchets?
Options: (a) hard-fail the gate on any increase; the only escape is a baseline bump in the same commit plus a `decisions.md` line; (b) warn only.
Recommendation: (a); warnings are ignored by twelve parallel lanes.
How to reverse it: switch the ratchet step to warn in `scripts/check.sh`.
Meanwhile: (a).

## Answered
