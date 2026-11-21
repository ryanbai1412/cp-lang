# PORTING.md

How to port an accepted solution. A port is evidence about the language, so
the bar is the exemplars, not "it passes the gate". The gold standard is
`design/EXEMPLARS-PORTS.md`: five real ports written as they should read.
Every dream must be at that level, and every port is scored against it.
`design/EXEMPLARS.md` and `design/sketches/` show more of the style.

## Order of work

1. **Understand the problem, not the C++.** Read the statement and the
   editorial idea. Check the original is complete and passes the samples
   (`scripts/pick-ports.py` filters drafts, but verify).
2. **Write the dream first:** `proto/ports/<group>/dream/NAME.cp`, before
   you look at what compiles. The program as it should read in the ideal
   CP Lang: invent syntax and library calls freely, in the exemplars' style.
   Not gated or formatted. If the dream reads like the C++, start over.
3. **Write the port** as close to the dream as today's language allows,
   gate it (`proto/ports/<group>/gate.sh NAME 150`), and measure tokens,
   runtime on a max input against the C++, and emitted bytes.
   A rewrite toward the dream may grow emitted C++ only when its review
   names the runtime part that grew and the codegen ticket that removes it;
   accept the growth by rebaselining the port in the same commit
   (`python3 scripts/scoreboard.py --update GROUP/NAME`), which the
   scoreboard's `reviewed rewrite` failure names; the 48 KB warn line
   still blocks.
4. **Report every gap** between port and dream in
   `research/frictions/NAME.md`: the snippet, the dream lines that remove it,
   the goal it hurts, a general fix (never a one-problem helper), severity.
5. **Self-review against the checklist below**, then commit port, dream,
   report and `research/reviews/NAME.md` holding `Score: pending` together:
   `port(NAME): <what the program does>`.

## Taste checklist

A port fails review if any of these hold and the gap is not reported:

- It does not read like the editorial: the invariant, recurrence or
  algorithm is buried in index bookkeeping.
- It hand-rolls a standard algorithm or structure (DFS for a tree path,
  diameter, a search loop with a `?T` flag where `find`/`first`/`any`
  belongs, a manual run-length loop).
- It keeps two copies of one fact in sync by hand (symmetric matrices,
  parallel arrays, sentinel bookkeeping).
- It discards results (`let _ =`) or threads state a closure could own.
- It mixes index spaces through raw `Int` where a domain would keep them
  apart, or pads with sentinels where a type or `?T` says it.
- Each fact is not in one place: the bound, base or modulus is repeated.
- It is slower than the C++ by more than 1.1x on max input without a
  performance friction filed.
- It drops a statement constraint today's types can hold
  (`let x: Int = read()` for `1 <= x <= 1e7`, `-1` sentinels for
  `-1 <= a_i < n`).
- A friction says the language lacks something without a run showing it
  fails: try the dream line in `cpl run` on a freshly built `cpl` first,
  and quote the error.

A dream fails review, and caps at 3, if it keeps the C++ construction where
the editorial names an object or rule, hides a side effect in an
expression (`first(xs, x => try_add(x))`), writes a loop or formula twice,
leaves a constant or closing formula the header does not derive, or mixes
spellings: `@!multitest` only, `Mod<p>`, `read<T>`, `struct`, methods for
sequence operations (`xs.sum()`, `xs.any(p)`, `xs.sorted()`).

## Review

Every new port gets a cold taste review: a fresh agent with only
`AGENTS.md`, this file, the exemplars, the statement, the dream and the
port. It scores the port 1 to 5 against the dream and lists the failures; a 5
reads like `design/EXEMPLARS-PORTS.md`. It also scores the dream itself: a
dream that reads like the C++ is sent back before the port is judged.
Ports under 4 go back to their author with the review before the wave
counts as done. Recurring findings become lane work or checklist lines.

The review lives in `research/reviews/NAME.md` (format in that directory's
README) and its score shows in `research/PORTS.md`. `scripts/check.sh
--fast` runs `scripts/review-check.py`, which fails when a port newer than
`research/reviews/baseline.json` has no dream or no review, and when a
commit adds a port while more new ports score under 4 than the ratchet in
the baseline allows. Lower the ratchet when a rewrite lifts a port; never
raise it.
