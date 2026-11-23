# Idiomatic CP Lang (draft 2)

Status: non-normative. This is taste guidance, not the language: `spec/core`, `spec/lib`,
`std` and `spec/tools` define CP Lang, and where this file disagrees with them, they win.
Whether it becomes Ryan's normative taste is open in `plan/QUESTIONS.md` (idioms). Changes go
through the idioms lane; a lint in `spec/lints/` should cite the practice it serves. Spellings follow `spec/` as of 2026-11-22.

Two parts. **Philosophy** is a handful of beliefs about what good contest code is; it changes
almost never. **Practices** are the concrete habits that follow from it, grouped by topic; each
names the belief it serves and the lint families that check it (`lints.md`). A practice that
serves no belief is a style preference we should not ship. A lint that serves no practice is
noise.

---

# Part 1. Philosophy

**B1. Say the idea.** The code reads like the editorial: the invariant, the recurrence, the
algorithm's name. Mechanism the reader already knows (how a Fenwick tree updates, how a binary
search narrows) belongs in a library, not in the solution.

**B2. Every fact once.** A bound, a modulus, an order, a transition or an output format is
written in one place and referenced everywhere else. When the idea changes, the edit is local.

**B3. Make the wrong program unwritable.** When you know two things differ (a vertex and an
edge, a position and a cut, absent and present, this test and the next), the code keeps them
apart, so the type checker catches the mix-up instead of the judge.

**B4. The obvious reading is the true one.** What looks O(1) is O(1); what looks like a read
does not mutate; what looks like a copy is a copy. The natural code is the correct and fast
code, and anything unusual (an escape hatch, a hidden cost) is visible where it happens.

**B5. Doubt is part of the program.** The brute force, the generator, the checker and the
invariants live beside the solution and are written early. "I think it's right" becomes
"stress agrees on 10,000 cases".

**B6. Clear beats short.** Fewer tokens is a means, never the goal. Two clear lines beat one
clever line; a named step beats a five-deep chain. This is the brake on every "shorter"
practice below.

The one-line test: **the code says the editorial's idea in the editorial's words, every fact
once, and the obvious reading is the correct and fast one.**

---

# Part 2. Practices

Each practice: the habit, a short example, then *Serves* (beliefs) and *Lints* (families in
`lints.md`).

## Algorithms and the library

**1. Name the algorithm.** If the idea has a name (prefix sums, DSU, Dijkstra, binary search
on the answer, sliding-window minimum, KMP), call it.

```
// hand-rolled                                 // idiomatic
var p = [0..=n] _ => 0                         let pre = Prefix(a)
for i in 0..n: p[i + 1] = p[i] + a[i]          print(pre.fold(l..r))
print(p[r] - p[l])
```

If the call needs three adapters to fit, the library is missing a shape: log it as a stdlib
bug and keep the loop until it is fixed.
*Serves* B1, B3. *Lints* S (stdlib replacements), H (C++ habits).

**2. Whole-sequence operations for reductions, searches and builds; loops for algorithms.**
A loop that only sums, counts, finds, tests all, takes the best or builds a list is `xs.sum()`,
`xs.count(p)`, `xs.first(p)`, `xs.all(p)`, `xs.max(by:)`, `[f(x) for x in xs]`. A simulation,
a two-pointer sweep with real state, or a union-find sweep stays a loop. Don't contort an
imperative algorithm into folds.
*Serves* B1, B6. *Lints* S.seq, F.

## Index spaces and ranges

**3. Name an index space when it is shared or confusable.** `domain V = 1..=n` once, then
`read<[V]Int>()` and `[V] v => ...` carry it. Name it when two arrays share it or when two
spaces could be mixed up (vertices and edges, rows and columns, raw and compressed). A lone
array over `0..n`, a bitmask over `0..2^n`, or a list that grows stays plain.
*Serves* B3. *Lints* D.

**4. Step positions; cut ranges.** A position steps with `p.next` / `p.prev` (`?V`), not
`p + 1`. A range is a pair of cuts (`l..r`, `p.before`, `p.after`), so prefix sums, slices and
binary searches never need `+ 1`. Convert to `Int` once, at the boundary where a number is the
meaning (output, `n - k`).
*Serves* B3. *Lints* D.

## Absence

**5. Absence is an optional, never a sentinel.** Not `-1`, `INF`, `0x3f3f3f3f`, `n` as "no
parent" or `0` as "unvisited". Write `?T`, seed with `None`, update with `min=` / `max=`, and
decide what absence prints at the print: `print(best ?? -1)`.

- `if x != None: ... x!` says one thing twice: `if x is Some(v):`.
- `!` asserts. Fine where the invariant is obvious and local; never the default way to read.
- A table filled completely by a loop becomes total once: `let dist = dist!`.

*Serves* B3. *Lints* O.

## Constraints and arithmetic

**6. Write each constraint once, as a type.** Bounds are declarations (`read<1..=2e5>()`,
`[V](a: Seg, b: Seg)`); the modulus is a type (`type Z = Mod<998244353>`). They drive
validation, storage width, overflow proofs, generators and the brute bound. The same bound
or modulus written as a literal twice is two facts that can disagree.
*Serves* B2, B3. *Lints* K, D.

**7. One home for each choice.** The order (`by:`), the transition, the output format: written
once, referenced. Mirror-image work (`add`/`remove`, `(u, v)`/`(v, u)`) is one function or one
symmetric structure. Revision test: when the DP gains a dimension or a search returns an end
instead of a start, the edit is local.
*Serves* B2. *Lints* R, N.

## Mutation and ownership

**8. Values by default; mutation is explicit and local.** `let` unless reassigned. `mut` at
the parameter and at every call site. Lend instead of cloning; `.clone()` only where a second
copy is meant. No aliasing tricks.
*Serves* B4. *Lints* W, M.

**9. Per-test state is declared per test.** Under `@!multitest`, everything that must reset is
declared below it; shared tables (sieve, factorials) above it, sized for the maximum once.
*Serves* B3. *Lints* T.

## Costs

**10. Costs show.** Anything O(n) or allocating looks like it. Lazy views are read once or
collected deliberately. Re-folding a window, sorting inside a loop, `in` on a `List`, a `@memo`
table bigger than memory, and Mod division by a loop invariant each get a lint. Never trade
correctness for speed by hand when a structure gives both.
*Serves* B4. *Lints* C, A.

## Input and output

**11. Print what you mean.** `print(yes(ok))`, not `1`/`0` or a hand-built ternary.
`print(xs)` with `sep:` / `end:`, not a join loop. `@!multitest`, not a `t` loop. Interactive
I/O through `ask` / `Wire`, which flush. Answer words are settings: `with io(answers: ("Yes",
"No"))` once, or `yes(ok, answers:)` at one call.
*Serves* B1, B4. *Lints* I.

## Evidence

**12. Write the doubt down.** `@alt` brute, `@gen` generator and `@check` checker in the same
file, early. Asserts state the invariant the editorial relies on. A solution that can't be
stress-tested against a brute isn't finished.
*Serves* B5. *Lints* X.

## Spelling and escape hatches

**13. One spelling per idea.** Methods on the value (`xs.sum()`), options as named arguments
(`by:`, `desc`) not `_by` twins, getters without `()`, punned arguments (`f(moves:)`), generic
calls `f<T>(...)`. The formatter settles layout; lints settle spelling.
*Serves* B1, B4. *Lints* N.

**14. Escape hatches are explicit, rare and local.** `!`, `Int(v)`, raw `[n]T`, `unchecked`,
inline C++: legal, greppable, used at the one place needed with the reason obvious. A hatch in
every solution of one shape means the language is missing something; log it.
*Serves* B4. *Lints* E.

---

## How this is used

1. **Lints** cite one practice (`#5`) and its belief in the message and docs page.
2. **The skill** teaches Part 1 in a paragraph, Part 2 one line each, then the rewrite table
   generated from autofixable lints.
3. **The site's idioms guide** is Part 2 with its examples, linking into the lint pages.
4. **Corpus review** asks of each line: which practice does it break? A break no lint catches
   is a candidate lint (see `lints.md`, "Adding a lint").

## Open

- #2 vs B6: where a fold chain gets long, the line between "sequence operation" and
  "contorted" needs examples from real ports, not a rule.
- #3: the exact trigger for the domain lint (one `Int` indexing arrays of different bounds;
  `± 1` on a position) needs a false-positive check on the corpus.
