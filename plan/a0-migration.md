# 10. Migration from C++

Status: proposed. Two jobs: let a C++ competitor bring their habits and
library over, and let agents port the corpus quickly. The user's rule
applies throughout: a port is a rethinking, not a translation.

## 10.1 `cp port`: a draft from C++

`cp port sol.cpp` writes `sol.cp`, a draft that compiles or nearly
compiles, with every place that needs a human decision marked.

It recognizes the common contest idioms:

- `#define int long long`, `using ll = long long` become `Int`.
- A `vector` sized `n` and indexed `0..n-1`, or sized `n + 1` and indexed
  `1..=n`, becomes a domain and a `[V]T` array. Arrays sized `n` and `m`
  that are indexed by the same loop variables get the same domain.
- Prefix arrays (`pre[i + 1] = pre[i] + a[i]`) become `[V.bounds]` arrays.
- `INF`, `-1` and `1e18` used as "no value" become `?T` with `none`.
- Template I/O (`cin >> n`, custom `read()` helpers) becomes typed `read()`
  using the constraints in comments if present.
- `vector<vector<int>> adj` built from an edge list becomes
  `Graph<V>.undirected(edges)` or `.directed`, from how edges are added.
- `for (int t = 0; t < T; t++) solve();` becomes file-level `@multitest`.
- Global arrays reset between tests become file-body `let`s, which are
  fresh each test.
- A segment tree class with a combine function becomes `SegTree<D, T>`
  with an operation value, when its shape matches.

What it doesn't do: line-by-line output for code it doesn't understand.
Unrecognized code becomes a `// PORT:` comment with the C++ beside it and a
best guess below, so the reader sees exactly what to rethink:

```
// PORT: manual index arithmetic; is this a domain step (v.next) or a value?
//   C++: if (i + 1 < n && a[i + 1] < a[i]) drops++;
for v in V:
    if let u = v.next and a[u] < a[v]: drops += 1
```

### Why it matters beyond users

Porting hundreds of corpus submissions is how the design is validated
(doc 8). A mechanical first pass makes that fast, and the `// PORT:` marks
are design feedback: an idiom that keeps needing manual rethinking means
either the tool or the language is missing something. The tool keeps a
tally per idiom.

### How it's measured

On the corpus: the share of drafts that compile after fixing only the
marked spots, and the edit distance between the draft and the final
reviewed port. Both are tracked over time.

## 10.2 `@cpp` interop

A `@cpp` block declares a C++ function with a typed signature, and its body
is pasted verbatim into the output:

```
@cpp fn popcount128(x: I128) -> Int:
    """
    return __builtin_popcountll((unsigned long long)x) +
           __builtin_popcountll((unsigned long long)(x >> 64));
    """
```

- **Boundary checks:** domain positions cross as checked integers, arrays
  as a pointer plus length, and nothing else crosses (no references kept
  past the call).
- **Debug runs:** the interpreter can't run `@cpp` code, so a file using it
  runs natively in debug mode, with a warning saying which tools are
  unavailable (trace, snapshots).
- **Purpose:** bringing over a snippet that hasn't been ported yet
  (decision 7.5). The linter counts `@cpp` blocks and suggests stdlib
  replacements where they exist.

## 10.3 Bringing over a personal library

Most strong competitors have a C++ library (the user's may be
`ryanbai1412/cp-library`). Migration has three parts:

1. **A map of equivalents.** For each file in the library, the stdlib item
   that replaces it, if any: "your `segtree.h` is `std.segtree` with
   `Monoid(...)`", "your `dsu.h` is `Dsu<V>`", "your `modint.h` is
   `ModInt<M>`". `cp port --lib DIR` generates this map and lists what has
   no equivalent.
2. **Porting what's left** into the user's `lib/` (doc 4), with `cp port`
   and review. These become library files with `@test fn`s.
3. **Keeping old C++ solutions buildable.** Nothing here touches them; the
   C++ files stay as they are.

## 10.4 Learning path for C++ users

The docs (doc 9) are written as "C++ habit, then the better way", and the
playground offers "paste your C++, get a draft". Every guide chapter ends
with a side-by-side port from the cookbook.
