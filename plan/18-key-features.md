# 1.9 Key features (proposed, easy to cut)

Status: proposed, folded into the plan but unreviewed. Each section can be cut on its own; docs 1.7, 1.8 and 4b refer to them as proposed. They came out of the second pass over
the compiler series. Each one would change existing designs, not just add
to them, so it's better to decide now. They're listed by expected impact.

## 1. Input types that drive everything

Today, typed reads give the compiler the input bounds:

```
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]1..=1e9 = read()
```

The proposal is to make input types expressive enough that the compiler can
also **generate**, **shrink** and **validate** inputs from them. Then
`cp stress` needs no generator code at all:

```
@multitest(sum: n <= 2e5)
let n: 1..=2e5 = read()
domain V = 1..=n
let p: Perm<V> = read()              // a permutation of V
let t: Tree<V> = read(edges)         // n - 1 edges forming a tree
let qs: List<(l: V, r: V) where l <= r> = read(count: Int)
```

From these declarations:

- **Reading.** Each type knows its input format.
- **Validation.** In local runs, an input that breaks its type (a repeated
  value in a `Perm`, a cycle in a `Tree`) is an error that names the line.
  This catches bad hand-made tests and generator bugs.
- **Generation.** `cp stress` produces random valid inputs: small sizes first,
  then edge shapes (paths, stars, all-equal values), then random ones.
- **Structure-aware shrinking.** When a test fails, the shrinker removes a
  query, removes a tree leaf, or lowers a value, and keeps only changes that
  still fail. Every shrunk input is still valid, which isn't true for shrinkers
  that don't understand the input's structure.
- **Cost estimation.** The input bounds come from the same declarations.

What changes in the design:

- Input types become a real stdlib feature: a `Read` + `Gen` + `Shrink`
  trait trio, with stdlib impls for `Perm`, `Tree`, `Graph(connected:)`,
  `Distinct`, `Sorted`, grids and strings over an alphabet.
- `where` refinements on records (`l <= r`) need syntax. In the compiler they
  are checked at read time and give facts to range analysis.
- The tooling docs' generator section becomes "override only when needed".

I think this is the biggest UX change available. Stress testing goes from
"write a generator, a brute and a script" to "write a brute".

## 2. Contracts: `requires`, `ensures`, `invariant`

```
fn query(st: SegTree<V, Int>, s: Range1<V>) -> Int
    requires st.built
    ensures result >= 0

fn two_pointer(a: [V]Int) -> Int:
    var l = V.first
    for r in V:
        while a[r] - a[l] > k: l = l.next ?? break
        invariant l <= r
```

- **Local builds** check contracts, and the trace shows the first one that
  failed. That points at a bug far earlier than a wrong answer would.
- **Range analysis** uses them as facts. `invariant l <= r` is exactly the
  relational fact intervals can't infer on their own, so
  more checks get removed and more overflow proofs succeed.
- **Judge builds** drop the checks. A fact that was only assumed and never
  proven is used only for warnings, never for removing checks, so a wrong
  contract can't silently break a submission.

What changes: fn signature syntax, and range analysis gains relational
facts.

## 3. Compile-time evaluation

```
const fact = [0..=20] i => if i == 0: 1 else: fact[i - 1] * i
const small_primes = List(p for p in 2..=1000 if is_prime(p))
```

Any `const` whose initializer doesn't read input is run by the interpreter
at compile time, and the result is embedded in the C++ as data. That covers:

- precomputed tables;
- "precompute offline, embed the answers" solutions, which are common for
  problems with a small input space;
- constant folding of user code beyond literals.

The 64 KB budget applies: the size report shows each embedded table, and
anything above about 16 KB is a warning.

What changes: `const` becomes "evaluated at compile time" rather than
"literal". The interpreter must be deterministic and sandboxed, with a time
limit. `static let` stays as the run-time alternative.

## 4. A REPL on a loaded test

```
$ cp repl d i2
loaded i2: n = 7, a = [3 1 4 1 5 9 2]
> answer()
5
> runs(a).count()
7
> :trace answer
```

The file is a function body (plan/15-program-structure.md), so loading a test means
running the body up to its last top-level `let`. After that, every top-level
fn and expression can be evaluated, using the interpreter and the same
snapshots the trace uses. The REPL can also open any saved snapshot.

What changes: very little in the language. It needs the interpreter to keep
the file body's environment alive and to accept new expressions against it.
I've listed it because it changes how debugging feels day to day.

## Recommendation

Adopt 1 and 2. Both change the design, and both strengthen features we
already have: 1 turns typed input into stress testing for free, and 2
fills the relational gap in range analysis. Adopt 3 with the size warning.
4 is cheap and can be added any time.
