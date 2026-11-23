# Taste reviews

One file per port, `NAME.md`, written cold: the reviewer reads `AGENTS.md`,
`handbook/process/porting.md`, the exemplars, the statement (or the friction report),
the dream and the port, never the C++ original. The bar is
`research/design-history/EXEMPLARS-PORTS.md`: a 5 reads like one of its exemplars, for the
port and for the dream. `scripts/review-check.py` reads the first `Score:`
line (the port) and `Dream:` line (the dream) and fails `scripts/check.sh --fast` when a
port newer than `baseline.json` has no dream or no review.

A port author commits `Score: pending` with the port; the reviewer replaces
it with a score.

## Scores

- **5** could be an exemplar in `research/design-history/EXEMPLARS-PORTS.md`: reads like the editorial, every fact once,
  nothing a contestant would want to change.
- **4** reads like the editorial; the gaps are minor and every one is in the
  friction report.
- **3** correct but translated, or a standard algorithm hand-rolled because
  the stdlib lacks it or is measured too slow, with the gap reported and
  measured. A port with no dream scores at most 3.
- **2** hand-rolled standard algorithms or bookkeeping the language already
  removes, or a fact kept in two places, unreported or reported with an
  excuse that does not hold.
- **1** unreadable.

## Dream score

`Dream: N/5` on the line after `Score:`, on the same scale, judged against
the exemplars in `research/design-history/EXEMPLARS-PORTS.md`: the header states the
invariant, standard algorithms are named, structures with an invariant get a
type, searches are `find`/`first`/`any`, index spaces are domains, each
bound once, no sentinels or fake monoids. A dream under 4 reads like the C++
and is sent back: the review says what the dream should say instead, and the
port is judged against that rewrite rather than the dream as written.

Calibrate against `research/design-history/EXEMPLARS-PORTS.md` (cf2147h, cf2084h, cf2147f,
cf2255e2) and `research/design-history/sketches/`. Any of these is a dream failure and
caps the dream at 3:

- the C++ construction kept (a greedy builder, a cycle walk, a pointer chase)
  where the editorial names an object or a rule;
- a side effect hidden in an expression (`first(xs, x => try_add(x))`, a
  `draw` that prints and returns an id);
- the same loop or formula written twice;
- an unexplained constant or closing formula the header does not derive;
- inconsistent spelling: `@!multitest` is the one multitest marker (not
  `--- multitest ---` or `--- with multitest() ---`); `Mod<p>`, `read<T>`,
  `struct`, `if c: a else: b`, `?? return`, `if` filters and `**` for power (decision §70); sequence
  folds and sorts of one materialised sequence as methods (`xs.sum()`,
  `xs.any(p)`, `xs.sorted()`), not `sum(xs)`/`any(xs, p)`/`sort(mut xs)`.
  Decision §53.1 keeps the free form canonical over a generator
  (`sum(f(x) for x in xs)`) and over two or more values (`min(a, b)`); never
  cite §53 against those.
- range ends: decision §101 makes `.start`/`.end` the accessors with `end`
  exclusive, so a 1-based inclusive pair printed as `start + 1, end` is
  correct, not off by one.
- pattern tests: decision §111 binds headers with `if let`/`while let`;
  `x is pat and cond` in a value position (a `Bool` argument or assignment)
  is canonical. Flag only `if x is p:` and `while x is p:` headers.

A dream under 4 goes back to its port's lane (the polish lane if that lane
has moved on) to rewrite before the port is judged: a new review of such a
port stays `Score: pending`, and an existing port score is marked
provisional.

## Sections

- **Checklist failures:** each failed `handbook/process/porting.md` item, with the line.
- **Owed rewrites:** gaps today's language can already close. The author
  owes these; each names the rewrite and, where it matters, the run that
  shows it works.
- **Language gaps:** gaps that need the language. Each must be in the
  friction report; missing ones are flagged.
- **Dream notes:** where the dream itself misses the exemplar bar.
