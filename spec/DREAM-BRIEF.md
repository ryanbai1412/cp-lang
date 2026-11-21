# Writing a dream

A dream is the solution to a problem as it should read in CP Lang: the
syntax and library we intend, not what compiles today. Dreams are the
acceptance tests of the spec (`spec/README.md`): the spec is done when
almost every dream checks against it. A dream is judged only on how good it
is to read. It is never compiled against today's implementation and has no
port beside it.

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
  (exclusive), `.len`, `.rev`. A half-open range of cuts (`D.bounds`)
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
`design/stdlib/` and `spec/std/` first and reuse an existing name when one
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
