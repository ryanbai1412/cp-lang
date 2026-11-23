# Friction Backlog — Scout Batch 37: Hard IOI, NOI and NOIP Contest Classics

Ten unported problems from the `ioi*`, `noi*` and `noip*` bucket were rethought in CP Lang under `proto/ports/scout_batch_37/`: `noi10p1`, `ioi15p1`, `noi14p5`, `noip18p4`, `ioi09p5`, `noip06p3`, `noip10p4`, `ioi20p2`, `ioi12p5`, `noip04p4`.

Each has a v2-spec file (`<problem>.cp`, written against `docs/one-page-cheat-sheet.md`, not compiled) and a buildable file (`today/<problem>.cp`, compiled through the unmodified `proto/cpc`). The buildable files agree with the original accepted C++ on 300 random inputs each (200 small, 100 medium, 3,000 total), with the port side under ASan and UBSan. The limits of that check are listed in the batch README; the main ones are that the v2 files are verified by reading only, that the three grader problems are compared through a stdin/stdout wrapper rather than an official grader or checker, and that nothing was run at maximum size.

All ten were written from the accepted C++, because the corpus carries no statements. None is statement-first.

## Dual LOC

Counted by `proto/ports/scout_batch_37/loc.py`: non-blank, non-comment lines. "Stripped" removes the contest template that opens every C++ file (includes, debug macros, aliases) and the stream setup lines, as if C++ already had a perfect library. None of these ten originals contains a reusable data structure, so the stripped figure is the template only.

```
problem      raw stripped   v2 today
noi10p1       66       42    9    13
ioi15p1       48       26    7    11
noi14p5      101       73   29    35
noip18p4     101       72   23    32
ioi09p5       70       42   19    26
noip06p3      94       66   20    24
noip10p4     132      104   30    39
ioi20p2      116       92   30    44
ioi12p5      101       77   42    42
noip04p4     101       73   34    37
total        930      667  243   303
```

The v2 ports are 74% shorter than the raw C++ and 64% shorter than the stripped C++. The buildable forms are 67% and 55% shorter. The 60-line gap between the two CP columns is the cost of the prototype gaps in section 2.

`ioi12p5` is the outlier: 42 lines in both forms. It is a two-program protocol with a set-based eviction replay, and nothing in the language shortens that; the algorithm is the code.

## 1. Friction and collisions in the v2 design itself

These remain even if every cheat-sheet feature is implemented.

### 1.1 `@grader` signatures have nowhere to put a shared domain

**Hit in:** `ioi20p2.cp`, `ioi12p5.cp`.

The cheat sheet shows one `@grader` function. Batch 25 established that a domain can be inferred from an argument's shape inside that function (`domain Colour, Ticket`). That stops working as soon as two grader declarations share a domain. In Connecting Supertrees the solution calls back into the grader with a matrix over the same towers:

```cp
@grader fn build(b: [V, V]Bool)        // V is not in scope here

@grader
fn construct(p: [V, V]0..=3) -> Bool:
    domain V                           // inferred from the shape of p
    ...
    build(bridge)
```

`build` is declared at file scope, but `V` only exists inside `construct`. Last Supper is worse: `Colour = 0..n` appears in five grader signatures (`ComputeAdvice`, `Assist`, `WriteAdvice`, `GetRequest`, `PutBack`), `n` arrives as a plain argument, and `ComputeAdvice` and `Assist` run in separate processes.

```cp
@grader fn GetRequest() -> Colour
@grader fn PutBack(c: Colour)

@grader
fn ComputeAdvice(req: [0..n]Colour, n: Int, k: Int, m: Int):
```

This is a collision between three features: nominal domains, shape inference, and extern grader declarations. The honest v2 spelling today is `Int` everywhere in the signatures plus `Colour(x)` casts in the body, which gives up the domain exactly where the judge hands over untrusted indices.

**Recommendation:** let a `@grader` file declare size-less domains at file scope (`domain V`, `domain Colour`) that are bound by the first grader call whose arguments fix them, and let every other grader signature in the file name them.

### 1.2 Array parameters sized by a later parameter

**Hit in:** `ioi15p1.cp`, `ioi12p5.cp`.

IOI signatures pass the length next to the array, often after it:

```cp
fn delivery(n: Int, k: Int, l: Int, pos: [0..n]Int) -> Int:
fn Assist(needed: [0..r]Bool, n: Int, k: Int, r: Int):
```

The cheat sheet does not say whether `[0..n]Int` in a parameter list may refer to a sibling parameter, or to one declared to its right. `proto/cpc` rejects it (`unknown name n`). The official signatures cannot be reordered, so this needs a rule.

### 1.3 A prefix of a domain has no spelling

**Hit in:** `ioi12p5.cp`.

"The first `k` colours" is a domain-typed loop over a prefix. Ranges over a nominal domain exist, but their ends must be domain values, and `k` is a count:

```cp
for c in Colour if c.to_idx < k:      // what the port uses: scans all n
for i in 0..k: let c = Colour(i)      // the alternative: a cast
```

Neither is the one obvious way. `Colour.first(k)` or `Colour[..k]` would be.

### 1.4 Row-major cells: one board, two views

**Hit in:** `noi14p5.cp`.

The board is shuffled as a flat sequence of `n * m` cells (swap cell `i` with cell `x % i`), then read as a grid (row and column of each value). v2 has `[R, C]T` and flat `[Cell]T` but no documented way to go between a flat cell and its `(row, col)`, so the port falls back to raw arithmetic, which is exactly the pattern product domains were meant to remove:

```cp
let (r, col) = (at[v].to_idx / m, at[v].to_idx % m)
```

**Recommendation:** document `to_idx` and `from_idx` on product domains as row-major, so `(R, C).from_idx(i)` and `p.to_idx` cover this.

### 1.5 Stepping to the next domain value at the end of a sequence

**Hit in:** `noip06p3.cp`.

Each job has a "next step" cursor that is advanced `m` times over a domain of `m` steps. The last advance has nowhere to go, so the natural `next[j] += 1` is not expressible and the port writes:

```cp
let s = next[j]
if let after = s + 1: next[j] = after
```

This is sound but reads as a workaround for a cursor that is simply never read again. A `?Step` cursor moves the noise to every read instead. The checked `v + k -> ?V` rule has no cheap form for one-past-the-end cursors.

### 1.6 Assigning through an optional record or tuple on backtracking

**Hit in:** `noip04p4.cp`.

Backtracking keeps two views of one assignment (`digit: [Letter]?Int` and `used: [0..n]Bool`) and every try sets and clears both by hand, three times in one function:

```cp
digit[l] = d
used[d] = true
if plausible() and solve(col, carry): return true
digit[l] = none
used[d] = false
```

There is no scoped "set these, run, restore" construct, and a helper cannot express it without stored borrows. This is the most repeated boilerplate in the batch.

### 1.7 Greedy interval cover re-evaluates a `@memo` call in a filter

**Hit in:** `noip10p4.cp`.

A generator that filters on one field and reduces over another needs the value twice:

```cp
covered = max(fed((R.first, c)).hi for c in C if fed((R.first, c)).lo <= covered)!
```

`@memo` makes it cheap, but it is still written twice. A `let` clause inside generators (`for c in C let f = fed(...) if f.lo <= covered`) would remove it.

## 2. Gaps between the cheat sheet and `proto/cpc`

Each of these is documented in v2, was written in the v2 file, and had to be rewritten for the buildable file. The error text is what `proto/cpc` printed.

`.step(k)` on a range is missing (`Range<Int> has no method 'step'`). The gcd sieve in `noi10p1` goes from one line to a four-line `while`:

```cp
pairs[g] = (n / g) * (m / g) - sum(pairs[mult] for mult in (2 * g..=k).step(g))
```

`MinHeap` has no `.len` (`MinHeap<Spot> has no '.len'`), and `Deque` does not exist (`unknown type Deque`). `ioi09p5` carries a separate `spots` counter and a `List` with a moving `head` index instead.

`while ... and let` is not parsed (`expected ':' (at 'and')`). The v2 queue drain in `ioi09p5`,

```cp
while not free.is_empty and let car = waiting.pop_front():
```

becomes a `while` on two counters with the pops inside.

Filtered loops, `for v in V if cond:`, are not parsed (`expected ':' (at 'if')`), although the cheat sheet lists them. Every one becomes a nested `if`, which is most of the extra lines in `ioi20p2`.

`Str` cannot be indexed (`cannot index Str`), as batch 30 already found. `noip04p4` reads `[0..3, 0..n]Char` instead.

`p.adj4` over a product-domain cell is missing; `noip10p4` writes the four offsets and the bounds test on `.ord` by hand.

`@grader` with an array sized by a parameter fails (`unknown name n`), so the three grader problems are top-level stdin programs in `today/`.

The `[D] v => expr` comprehension does not extend to two anonymous ranges: `[0..3, 0..n] (i, j) => i + j` fails with `` `+` between range and range ``.

Two rules disagree on optionality. The cheat sheet gives `V.from_idx(i) -> ?V`; the prototype returns a plain `V` and rejects the `!` (`` `!` on a non-optional Cell``). And `min(expr for i in 0..n)` over a possibly empty range returns a plain `Int` in the prototype (`` `!` on a non-optional Int``), where the unified-optional rule in v2 implies `?Int`. Whichever is intended, the cheat sheet should say what a reduction over an empty generator returns.

Things that did work first time and are worth keeping: nested functions capturing locals, `@memo` returning a named record, comparing `?Int` against an `Int`, `min=` on `?List<Int>`, and lexicographic `<` on lists.

## 3. Range checker

**Hit in:** `noi14p5.cp`.

The generator recurrence is `x = (a * x * x + b * x + c) % d`. With the bounds first written for `a`, the build stopped with `` `a * x * x` may reach 1e+27, beyond Int``. The port declares `a: 0..=300`, which makes the product fit, and the test generator stays inside it. That bound was chosen to satisfy the checker, not read from a statement, so the real limit on `a` should be confirmed before this port is treated as final. If the real bound is larger, the port needs an explicit modular multiply and the language needs an obvious one (`mul_mod(a, b, d)`), since `Mod<P>` does not apply to a runtime modulus without `ModR`.

The message itself was good: it named the expression and the magnitude.

## 4. What read well

`noi10p1` and `ioi15p1` are 9 and 7 lines in v2 and say the algorithm directly; the reverse range and generator `min` carry them. `noip18p4`'s `tour(skip: ?(V, V))` with a nested `walk` replaces a solver struct and globals in the original. `noip10p4`'s `@memo fn fed(p: Cell) -> (lo:, hi:)` replaces a hand-written visited array and two result grids. In `ioi20p2`, `root: [V]?V` and `ring: [V]?V` make "not yet grouped" a type rather than a `-1`.

## 5. Ranked recommendations

1. Give `@grader` files file-scope domains bound by grader arguments, and define array parameters sized by sibling parameters (1.1, 1.2). Every IOI function-style task hits this.
2. Implement the documented loop forms in the prototype: `for ... if`, `while ... and let`, `.step(k)`, `Deque`, heap `.len` (section 2). They account for most of the 60-line gap.
3. Document row-major `to_idx`/`from_idx` on product domains and add a domain prefix form (1.3, 1.4).
4. Settle the optionality of `from_idx` and of reductions over empty generators (section 2).
5. Consider a scoped set-and-restore form for backtracking and a `let` clause in generators (1.6, 1.7). These are additions, so they should go through the concept-budget audit first.
