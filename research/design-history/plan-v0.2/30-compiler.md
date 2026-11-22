# 3. The compiler: what it does for you

Status: decided unless marked *proposed*. This doc describes what the
compiler delivers to the person using it, at full ambition. How it is
built is up to the builder.

## 3.1 Two ways to run every program

- **Debug runs** (`cp run d i1`) start in tens of milliseconds, with no
  g++ wait. Every index, range, overflow, borrow and law is checked, every
  value is recorded for the trace and snapshots (doc 5), and every runtime
  error is reported in source terms.
- **Builds** (`cp build d`) produce one C++17 file for the judge (doc 7):
  under 64 KB, deterministic, readable, stamped with versions, and as fast
  and as small in memory as what a strong competitor writes by hand.

Both give identical output on every program. That is a guarantee, tested
continuously (doc 8).

## 3.2 Speed of the tools

- The front end (all errors and warnings) answers in under 500 ms on any
  solution, and in tens of milliseconds after an edit in the editor, where
  only what changed is rechecked.
- The editor and the command line are the same compiler, so they never
  disagree about an error.
- Broken code is normal: a file with a syntax error still gets every other
  error, hover and completion.

## 3.3 What the generated C++ is like

The obvious code compiles to what you'd write by hand (G1, G2):

- No hidden copies, allocations, virtual calls, `std::function` or boxing.
- Generators, sequence views, operation values and domains disappear:
  a `for` over `runs(a)` is the hand-written loop; `SegTree<V, Int>(Max)`
  is a specialized segment tree with `max` inlined.
- Values take the fewest bytes their types allow: domain positions are 4
  bytes, `?V` and `?0..=1e9` are 4, `?Int` is 8, fields take the width of
  their range, records are packed in declaration order unless a
  reordering is never worse.
- Index checks: none on domain indexing; `List` and `raw` indexing checked
  unless proven or hoisted out of the loop; `cp build --checks` lists the
  ones left in loops; `unchecked:` drops them for one hot loop in the judge
  build only.
- Unused struct fields, functions and stdlib code are removed, and
  instantiations with the same layout share code, to protect the 64 KB
  limit. `cp explain size` shows what takes the bytes.
- No undefined behaviour the judge's optimizer could exploit.

## 3.4 What the compiler proves and rejects

- Types, with inference so annotations are only needed on fn signatures and
  where a literal is ambiguous.
- Domains: indexing with the wrong domain, mixing positions and bounds,
  and arithmetic on positions (doc 1.3).
- Ownership: two `mut` borrows of one value, a borrow outliving its owner,
  mutation while iterating, a missing `mut` at a call site (doc 1.5).
- Exhaustive `match`, no uninitialized reads, no reading a file-level `let`
  before it's set.
- Trait bounds checked at the generic definition, not at each use.
- Laws (associativity, monotonicity) checked by tooling at run time
  (`cp fntest`, debug runs), never assumed silently.

## 3.5 Value ranges and overflow

Every integer has a range, from its declared type, input constraints and
the arithmetic that produced it. The compiler propagates ranges through
`+ - * / %`, `min`, `max`, shifts and loops, and uses them for:

- **Overflow.** A value that provably can exceed `Int` is an error naming
  the expression and the inputs responsible:

```text
error: `total += a[i] * b[i]` can reach 2.0e23 (> 9.2e18)
       a: 0..=1e9, b: 0..=1e9, loop over V (len <= 2e5)
help:  declare `total: I128`, or tighten a bound with `assert`
```

  Proven safe: no check. Unknown: trapped in debug runs, unchecked in the
  judge build (*proposed*, decision 7.2).
- **Storage widths.** `[V]0..=1e9` is 4 bytes per element.
- **Removing index checks** that ranges prove.
- **Hover.** Every value shows its proven range.

When a fact is beyond the analysis ("the sum of `a` is at most 1e9"),
declare it in the input type or with `assert`; both also feed generators.

Evidence for why this matters, from the user's own rejected-then-accepted
diffs: `int x` to `long x` (sac22cc4p1, cco17p3), `int acnt[26]` to `long`
(mccc3j3), `stoi` to `stoll` (noip00p2). `Int` being 64-bit removes these;
range analysis catches the rest, the products that overflow 64 bits.

## 3.6 Cost estimates

Before you submit, the compiler estimates time and memory at the maximum
input and warns, never errors (the user's call):

```text
warning: this loop runs about 4e10 times (n * q, n, q <= 2e5); limit 1 s
   --> d.cp:31
warning: dp takes 3.2 GB ([V, V]Int, n <= 2e4); limit 256 MB
```

## 3.7 Diagnostics

Goal G5. Every message:

- names the value, what was expected, and why it was expected;
- says what to type next, with a machine-applicable fix when there is one;
- appears once per root cause: one missing `)` or one wrong type never
  cascades;
- has a code and an error-index page (doc 9);
- in generated or desugared code, points at what the user wrote (`in code
  generated by @memo on line 4`).

Runtime errors in debug runs name the source line, the values involved with
their domains, and the call path, never a C++ line or an address.

## 3.8 Versions

Every output records the compiler version and library snapshot hashes, so
any old solution can be rebuilt exactly (doc 11).
