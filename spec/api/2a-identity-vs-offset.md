# Index identity vs offset

What does `w[i]` mean when `w` is a slice, and what index does a search
inside a slice return? This doc lists the cases contest code actually hits,
scores each option on every case, and picks the option that does best.
Examples cite the gold solutions and dreams where possible.

## The options

| | Domain slice `a[l..=r]` of `[V]T` | `List`/`Str` slice `s[l..r]` |
|---|---|---|
| **I. Identity everywhere** | keeps `V`'s positions | keeps the source's `Int` keys `l..r` |
| **O. Offset everywhere** | renumbered `0..len`, keyed by `Int` | renumbered `0..len` |
| **H. Hybrid** (current round 2 text) | keeps `V`'s positions | renumbered `0..len` |

Scores in the tables:
- ✓ works as written
- \+ needs extra code that is easy to forget (an `+ l` or `- l`)
- T wrong code traps, at compile time or on the first wrong access
- S wrong code reads the wrong element and keeps going

S is the score that matters. The design rules rank a compile error over a
trap, and a trap over silent wrongness.

## A. Order-only uses: no key is ever observed

Most slice uses never look at a key, so the choice doesn't affect them.

| Case | Example | I | O | H |
|---|---|---|---|---|
| A1 walk, fold, print | `print(perm[lo..hi])`, `w.sum()` | ✓ | ✓ | ✓ |
| A2 compare two slices | `tuples[i - 1][..cl] != t[..cl]`, `b[j..].starts_with(a[span[s]])` | ✓ | ✓ | ✓ |
| A3 concatenate, collect | `p[..2 * skip].even + p[2 * skip..]` | ✓ | ✓ | ✓ |
| A4 zip slices at different offsets | `zip(ord[n - m..].rev, ord[k..n - m].rev)` | ✓ | ✓ | ✓ |
| A5 assign into a range | `q[start..] = suffix`, `dp[mid..hi] += dp[mid..hi.prev]` | ✓ | ✓ | ✓ |
| A6 in-place verbs | `a[l..=r].sort()`, `.reverse()` | ✓ | ✓ | ✓ |
| A7 consume from the front | `rest.first`, `rest = rest.drop(1)` | ✓ | ✓ | ✓ |
| A8 copy out | `List(w)`, `Str(s[l..r])` | ✓ | ✓ | ✓ |
| A9 length-only results | `period(t[start..start + delta])` | ✓ | ✓ | ✓ |

This group only works under every option because of one rule: **any
operation that combines two sequences pairs them by walk order, never by
key.** That covers `==`, `<`, `starts_with`, `zip`, `+`, slice assignment
and elementwise `+=`. The rule holds regardless of which option wins, and
round 2 should state it.

## B. An index found inside the slice, used outside it

| Case | Example | I | O | H (domain) | H (List/Str) |
|---|---|---|---|---|---|
| B1 arg-extreme in a range | D&C: `let m = a[l..=r].argmax()`, then `b[m]`, recurse on `l..m` | ✓ | + / S | ✓ | + / S |
| B2 search in a range | `a[l..r].bsearch(x)`, `s[l..].find(c)` | ✓ | + / S | ✓ | + / S |
| B3 update parallel arrays | `for (i, x) in a[l..=r].items: dp[i] = …` | ✓ | + / S | ✓ | + / S |
| B4 prefix of a range | `a[l..=r].scan(+)[c]` with a source cut `c` | ✓ | + / S | ✓ | + / S |

C++ itself answers B with identity: `max_element(a + l, a + r) - a` and
`lower_bound(a + l, a + r, x) - a` give absolute positions. Python's
`s.find(c, l)` is absolute too. Only Python's `s[l:].find(c)` is relative,
and adding `+ l` to it is a well-known Python bug.

## C. An outside index used on the slice

| Case | Example | I | O | H (domain) | H (List/Str) |
|---|---|---|---|---|---|
| C1 source position into a view | `blk[i]` inside a D&C on `blk = a[i.l..end]` | ✓ | + / S | ✓ | + / S |
| C2 nested slicing | `w[a..b]` with `a`, `b` from the source | ✓ | + / S | ✓ | + / S |
| C3 a range cut into halves | `let (x, y) = r.halve()`, then `a[x]`, `a[y]` | ✓ | + / S | ✓ | + / S |

## D. Counters and literals into the slice

| Case | Example | I | O | H (domain) | H (List/Str) |
|---|---|---|---|---|---|
| D1 literal first or last | `w[0]`, `w[w.len - 1]` | T | ✓ | T (compile) | ✓ |
| D2 counter loop | `for i in 0..w.len: w[i]` (festival's `rest[i]`) | T | ✓ | T (compile) | ✓ |
| D3 relative access by variable | `s[l..][k]`, meaning the k-th char after `l` | S | ✓ | T (compile) | ✓ |

Why identity scores T on D1 and D2 rather than S: the wrong counter starts
at 0. If the slice starts at 0, identity and offset agree, so the code is
correct. If it starts later, the first access `w[0]` is outside `w.keys` and
traps. Only D3, a variable offset into a slice that happens to land inside
the slice's keys, is silent. In practice D3 is written `s[l + k]`.

## E. Generic algorithms run on a sub-sequence

| Case | Example | I | O | H (domain) | H (List/Str) |
|---|---|---|---|---|---|
| E1 a fn written with `xs[0]` / `0..xs.len` gets a slice | user `fn f(xs: List<Int>)` | T | ✓ | T (compile) | ✓ |
| E2 a per-element table from an algorithm on a slice | `let z = z_function(s[l..])`, then `z[i]` for `i` in `s[l..].keys` | ✓ if keyed like the input; S if 0-based | ✓ | ✓ if keyed like the input | ✓ |

E2 is identity's real hazard. It goes away if every per-element result
**is keyed like its input** (`z_function(s: S) -> [S.keys]Int`). Arrays
over an `Int` range already exist (festival writes `[0..=rest.len]`), so
this is a stdlib convention, not a new feature.

## F. Grids and multi-dimensional arrays

| Case | Example | I | O | H |
|---|---|---|---|---|
| F1 row or column view | `grid[r]`, `grid[r, ..]`, `b[.., bv]` | ✓ (keeps the other domain) | S (row loses `C`) | ✓ |
| F2 sub-rectangle fold | `inside[..n, ..m].sum()` | ✓ | ✓ | ✓ |

## G. Revision: changing your mind

| Case | Change | I | O | H |
|---|---|---|---|---|
| G1 array ↔ list | `[V]Int` becomes `List<Int>` | same meaning | same meaning | slices change meaning; mostly caught, because `V` indices become type errors |
| G2 add `.rev` | `a[l..r]` becomes `a[l..r].rev` | `w[i]` is the same element | `w[0]` is now the last element | domain same; List/Str flip |
| G3 widen or narrow a slice | `s[l..r]` becomes `s[l - 1..r]` | every index still valid | every offset shifts by one | domain same; List/Str shift |

## H. Keys of the other views

| View | I | O / H (List/Str) |
|---|---|---|
| `.drop(k)`, `.take(k)` | source keys | renumbered |
| `.rev` | source keys, walked backwards | renumbered from the far end |
| `.even`, `.step(k)` | source keys, strided | renumbered `0, 1, 2…` |
| `.map(f)` | source keys | renumbered |
| `.filter(p)`, generators | no keys (`Seq`) | no keys |
| `.enumerate()` | `(counter, x)`, counter from 0 | `(counter, x)` |
| `.items` | `(key, x)` | `(offset, x)` |

## Tally

| Group | I | O | H |
|---|---|---|---|
| A order-only (most uses) | ✓ | ✓ | ✓ |
| B index out, C index in | ✓ | silent | ✓ domain, silent List/Str |
| D counters and literals | traps, except rare D3 | ✓ | ✓ |
| E generic algorithms | traps, ✓ with keyed results | ✓ | ✓ |
| F grids | ✓ | breaks | ✓ |
| G revision | stable | unstable | unstable for List/Str |

- **O** is silently wrong on B, C and F, which are core CP shapes. Out.
- **H** is safe on domain arrays but silent on B and C for `List`/`Str`. It
  also makes the same expression mean different things depending on the
  source's key type, which shows up in G.
- **I** fails loudly almost everywhere it fails. Its one silent case, D3, is
  rare and has an obvious spelling (`s[l + k]`). Its cost is unfamiliarity:
  `s[2..5][0]` traps, where Python returns `s[2]`.

## Recommendation: identity everywhere (I)

This reverses the hybrid that round 2 currently states. The principle: **an
index names the same element everywhere it is valid.** A slice narrows which
indices are valid; it never renames them. That holds for domain positions
and for `Int` keys alike, so there is one rule.

What comes with it:
1. **Order-based pairing** for every operation that combines two sequences
   (group A).
2. **`.items` gives keys; `.enumerate()` gives a 0-based counter.** Use
   `.items` when the index goes back into a sequence.
3. **`w.rebase`**: an O(1) view of `w` keyed `0..w.len`, for the rare
   relative case (D3, or E1 with a fn written 0-based). It allocates nothing.
4. **Per-element results are keyed like their input** (`[S.keys]T`), so
   `z_function(s[l..])[i]` works with `i` from `s[l..]`.
5. **Diagnostics:** a literal index into a slice is a lint ("`w` starts at
   `l`; use `w.first` or `w.rebase[0]`"). An out-of-keys access traps naming
   the slice's key range.
6. **Copies count from 0:** `List(w)`, `Str(w)` and `.collect()` build fresh
   sequences, keyed from 0.

## Open

1. Should `w.rebase` exist, or should the D3/E1 path be a copy
   (`List(w)`)? An O(1) view costs nothing, so I lean towards having it.
2. Does `Str` slicing stay key-preserving in printed output and hashing?
   Yes: printing and hashing are order-only (A1, A2).
3. Does `.enumerate()` survive at all? It duplicates `.items` on a fresh
   `List`. I lean towards keeping it for counters over generators.
