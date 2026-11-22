# Writing a dream

A dream is the solution to a problem as it should read in CP Lang: the
syntax and library we would most want, not what compiles today. Each
hill-climb problem in `corpus/problems/<problem>/` has two files:

- `dream.cp`: the ideal version, written blind from the C++ and the
  editorial idea, **without reading the spec or running the checker**. It
  is allowed to be wrong about the spec; that is the point.
- `spec.cp`: the spec-compliant solution, written after comparing the dream
  with the spec, and checked with `cpl check` against `spec/std`.

Every gap between the two is classified in `spec/delta/hillclimb.md` as a
spec change (a general mechanism, logged in `decisions.md`), a rejected
wish (with the reason; one-problem helpers are rejected), a dream bug, a
parser/checker gap (`spec/delta/parser-gaps.md`), or an idiom fix to
`spec.cp`. `meta.json` carries separate `dream` and `spec` statuses.
Writing a dream against the stubs until it checks merges the two steps and
hides friction; don't. `spec/delta/dream-comparisons.md` shows ten pairs side
by side.

## The bar

Read `design/EXEMPLARS-PORTS.md` first: each section shows a weak port next
to the exemplar for the same problem. Every dream must read like the
exemplars. A grader scores each dream 1 to 5; 5 is exemplar level, and
anything under 4 comes back with notes.

What a 5 has:

- A header comment of two to five lines stating the editorial idea: the
  invariant, recurrence or reduction. The code then reads as that argument,
  top to bottom.
- Standard algorithms and structures are named (`dijkstra`, `gomory_hu`,
  `SegTree`, `runs`, `Dsu`), never re-derived inline.
- A structure with an invariant gets a name and its operations instead of
  parallel arrays kept in sync by hand.
- Searches are `first`/`find`/`any`/`count`, not flag loops.
- Index spaces are domains (`domain V = read(1..=n)`), statement constraints
  are types (`let k: 1..=1e9 = read()`), and each bound, modulus and base is
  written once.
- No sentinels, no fake identities, no side effects hidden in expressions.
- It is short because it says the idea, not because it is clever.

What caps a dream at 3: it keeps the C++ construction where the editorial
names an object or rule, writes a loop or formula twice, leaves a constant
the header does not derive, or mixes spellings.

## Rethink, don't translate

Solve the problem from the statement, then ask how the solution is best
said. The best program may look nothing like a C++ solution. The algorithm
must be correct and fast enough for the limits; if you are unsure of the
solution, say so in the header rather than guessing silently.

## Spellings that are decided

`decisions.md` is authoritative; the ones dreams use most:

- `@!multitest` for multitest input; `read()` and `read<T>()` with
  constraint types; `print(...)`.
- `0..n` half-open, `0..=n` inclusive; ranges have `.start`, `.end`
  (exclusive), `.len`, `.rev`. A half-open range of cuts (`D.cuts`)
  converts to the piece of positions between them.
- `and`/`or`/`not`, `elif`, `if c: a else: b` as the inline conditional,
  `x is Pat`, `if let`/`while let`, `let ... else`, postfix `?`, `??`.
- `**` is power, `^` is xor; division and `%` floor.
- `x op= y` is a statement; `min=`/`max=` return whether they improved:
  `if dist[v] min= d: parent[v] = u`. Bump-and-read is `let k = mut c[x]`.
- Chained comparisons `lo <= x < hi` (one direction only).
- Getters are properties: `xs.len`, `xs.first`, `xs.last`, `xs.rev`;
  folds and sorts are methods: `xs.sum()`, `xs.count(p)`, `xs.sort(by: k)`.
- Options are named arguments (`by:`, `from:`), never `_by` variants.
- Loop-shaped library code is a generator used with plain `for`; `break`,
  `return`, labelled `break 'outer` and loop `else` work in it.
- Lambdas: `x => e`, multi-line bodies after `=>`, and a `let`-bound lambda
  may call itself.
- Returning a container moves it; `a.clone()` copies; `m[k] += 1` inserts
  the default for a missing key.
- Generic calls are `f<T>(...)`.

## Inventing library

You may use any stdlib name you wish existed. Check `stdlib/`,
`spec/lib/` and `spec/std/` first and reuse an existing name when one
fits. Every name you invent is listed in your wishes file with its full
signature, a one-line doc and its cost, so the spec can adopt it:

    pub fn suffix_mex(self: List<Int>) -> List<Int>   // mex of each suffix; O(n)

Prefer a general facility over a one-off helper: a helper with one use
means the general mechanism is missing, so propose that instead. New core
syntax is allowed only when no library facility can say it; list it under
"syntax" in the wishes file with the reason.

## Deliverables

For each problem NAME:

- `proto/ports/ecnerwala/dream/NAME.cp`: the dream.
- `spec/wishes/NAME.md`: every invented library name (signature, doc,
  cost) and every syntax wish (snippet and reason), or "none".

`scripts/spec-check.sh proto/ports/ecnerwala/dream/NAME.cp` shows what the
spec does not cover yet. It is information for the wishes file, never a
reason to make the dream worse.

The history is public: no internal hostnames, mirrors, session links,
secret names, credentials or personal paths.
