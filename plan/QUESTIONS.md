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

## Answered
