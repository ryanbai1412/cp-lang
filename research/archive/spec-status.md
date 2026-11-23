# Where 0.3.0 stands

The current state of the 0.3.0 work, for an agent picking it up cold. Update
this file whenever a phase moves or an open question closes. Last updated
2026-11-21, on `main`.

## The plan and how far it got

Ryan's plan, in order (his words are in `plan/agent-context.md`):

1. **Write the whole spec.** Done as a draft: `spec/README.md` indexes 14 core
   chapters, the library chapters in `lib/`, the tool contracts in `tools/`,
   and about 1,700 documented signatures in `std/` (bodies are `todo`).
2. **One independent review.** Done: `research/spec-review-0.3.0.md`. Its findings were
   folded in and logged in `decisions.md`.
3. **Hill-climb against 100 problems.** Done once over all 100 in
   `corpus/hillclimb.json`. Each has a blind `dream.cp` and a `spec.cp`
   (see "The loop" below). The changes this forced are logged in `decisions.md` (157 to 163 are the latest),
   and each is one line in `delta/hillclimb.md`.
4. **Spec-compliant solutions.** All 100 `spec.cp` files are drafts. 99 pass
   `cpl check` against `std/`; dmpg16g2 fails only on `Letter.subsets`. None
   has run against judge tests yet, because the library is stubs.
5. **Build out every domain** (compiler, library, LSP, tools). Not started for
   0.3.0. The 0.2.x compiler on this branch accepts most of the new syntax in
   the checker, but the interpreter and C++ lowering lag. See `delta/0.2.md`
   and `delta/parser-gaps.md`.

## Independent rebuilds

The spec is done when a fresh agent reading only `spec/` writes contest code
and answers compile/output questions with few guesses.

- **Round 1** (`delta/rebuild-1/`): 57 holes, all fixed (decisions
  179–186; the list is at the end of its `HOLES.md`).
- **Round 2** (`delta/rebuild-2/`): running. Target: fewer than 5 holes.
- `GLOSSARY.md` fixes one term per concept; `scripts/spec-vocab.py` (in
  `scripts/spec-check.sh`) fails on banned spellings.

## The loop

- `dream.cp` is written blind from the accepted C++ and the editorial idea,
  without reading the spec or running the checker. `spec.cp` is written after
  comparing the two, and must pass `cpl check`. Never write a dream against
  the stubs. The brief is `plan/DREAM-BRIEF.md`.
- Each gap between the pair is one of: a spec change (a general mechanism,
  added to `decisions.md` and the stubs), a rejected wish (one-problem helpers
  are rejected, with the reason written down), a dream bug, a checker gap
  (`delta/parser-gaps.md`), or an idiom fix to `spec.cp`.
- Read the pairs side by side in `delta/blind-dreams-first-ten.md` (the first
  blind batch) and `delta/dream-comparisons.md` (ten more, with the port and
  a line-by-line verdict).
- Rank ports by `spec.cp`/`dream.cp` length to find friction. The median ratio
  is 1.13x. Ports far above that usually have a missing general mechanism
  (`buckets`, `pieces`, `closure`, `try_from_parents` all came from there) or
  a port that ignores an accepted form.

## Checks

Run from the repo root:

```
scripts/check.sh --fast                                   # the gate, under 180 s
scripts/spec-check.sh                                     # gold and port dreams vs the stubs
scripts/spec-stub.py --check                              # every std body is `todo`, documented
scripts/corpus.py build && scripts/corpus.py check        # corpus records
target/debug/cpl check corpus/problems/<p>/spec.cp
```

The pre-commit hook runs a
public-history scan, so keep hosts, home paths and session links out of
commits.

## Settled tonight (2026-11-21)

| # | Decision |
|---|---|
| 157 | Enum variants are PascalCase (`Some`, `None`, `Anchor.Start`); in a pattern, a capitalised name is a variant and a lowercase name binds; only `true`/`false` stay lowercase |
| 158 | `a.buckets()`: `[D]K -> [K]List<D>`, the domain-keyed grouping; `group(by:)` stays hash-keyed |
| 159 | `(V, V)` is an inline product domain; a k-parameter lambda unpacks a k-tuple |
| 160 | `d.closure(over:)` is the value form of `floyd_warshall`, as `sorted` is to `sort` |
| 161 | `read_all(until:)` for sentinel-ended input; `group(by:, of:)` projects the grouped values |
| 162 | `D.pieces(cuts:)` and `piece.covering(r)`: coordinate compression for intervals |
| 163 | `Rooted.try_from_parents`: `?Rooted` for parent input that may have a cycle |
| 164 | `Pair<V>`: the unordered-pair domain, packed, read as a lower triangle, `d[u, v] == d[v, u]` |
| 165 | `read`/`print` signatures in chapter 10.5; output mirrors input shapes; `wire.ask("? {x}")` is one interactive query |

## Open

- **Conformance:** `just conform` (`cargo run -p cpl-conform -- conformance`) runs `spec/conformance`; every
  case the checker does not meet is a checker or interpreter bug against the
  spec (wrong diagnostic code, a lint reported as an error, a missing error).
  `--rules` reports rules with no case once chapters carry `(Rnn.n)` ids.
- **Library docs to the entry standard** (`std/README.md`): every pub fn states
  `cost`, `traps` and an `example`, plus `empty`, `ties` and `order` where they
  apply. `scripts/spec-stub.py --audit` lists the gaps (all 1,972 entries at
  the start); `--check` ratchets on `std/doc-baseline.json`. Converting, most
  used modules first (`seq`, `graph`, `tree`, `segtree`).
- **Other triangle shapes.** `Pair<V>` (164) covers the lower triangle
  without its diagonal, which is both corpus cases. An upper triangle (row
  `i` lists every later `j`) walks the pairs in a different order, and a
  triangle with its diagonal is not `Pair<V>`. Neither shape is in the
  corpus yet; decide when one appears.
- **The checker is loose on `[Pair<V>]T`.** It accepts the ports, but it also
  accepts a misspelt method on such an array, and `Pair<V>.count` fails.
- **`D.subsets`** is decided but not in the checker (dmpg16g2).
- **The checker is laxer than the spec.** It accepted `(V, V)` and unpacking
  lambdas before the spec defined them, so passing `cpl check` does not show
  the spec allows a form. Needed: a conformance pass that makes the checker
  reject undefined forms (`conformance/` holds the must/must-not examples).
- **Would another agent build the same language from `spec/`?** Not yet.
  Library semantics (ties, empty inputs, output order) are thin in many stubs,
  and contradictions still turn up (variant casing was one). The test: an
  agent that reads only `spec/` writes about 10 unseen problems and answers
  about 40 "does this compile, and what does it do" questions, and every
  disagreement is a spec hole. Not run yet.
- **Dream independence.** The same agent wrote the dreams and the spec, which
  probably understates friction. The next batch should come from an agent
  that has not read the spec.
- **btoi18p4's port** checks but is too slow and gets the wrap-around pair
  wrong.
- **Running the new forms:** the checker accepts `@tag` variants, enum and
  product-domain `.at`, and `(mut d, e) =>` lambdas; the interpreter and C++
  don't run them yet.
- **Whether `x is P(a)` binds in later comprehension clauses**
  (`[x for c in cs if c is Set(a, b) for x in [a, b]]` fails today; ioi05p2
  uses `flat_map` instead).

## Conventions picked up tonight

- An edge with one weight names it `w`, the implicit weight. An edge with
  several uses meaningful names and selects one with `.weighted(.len)`.
- The `try_` prefix marks the optional-returning form of an operation whose
  plain form assumes valid input (`try_inv`, `try_fold`, `try_from_parents`).
  `V.at(x)` / `V(x)` is the same split for conversions.
- A value-returning and an in-place pair is named like `sorted`/`sort`
  (`closure`/`floyd_warshall`).
