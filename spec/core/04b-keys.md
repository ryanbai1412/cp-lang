# Keys: identity everywhere, keys in, keys out

Follows `04a-identity-vs-offset.md`, which scored the cases and picked
identity everywhere. This doc shows how that choice looks in contest code,
in user helpers and in library code.

## The rules

1. **An index names the same element everywhere it is valid.** A slice
   narrows which keys are valid and never renames them. This holds for every
   source: domain arrays, `List` and `Str`. `s[2..5][3]` is `s[3]`, and
   `s[2..5][0]` traps. (R04b.1.1)
2. **Keys in, keys out.** A function given an indexed sequence answers in
   that sequence's keys:
   - a position it finds is one of the input's keys (`?K`), valid on the
     source;
   - a per-element table is keyed like the input (`[s.keys]T`);
   - a prefix table is indexed by the input's cuts. (R04b.1.2)

   Lengths, counts and periods stay `Int`.
3. **Combining two sequences pairs them by walk order, never by key.** This
   covers `==`, `<`, `starts_with`, `+`, slice assignment and `zip`.
   `zip(a, b)` is keyed by `a`'s keys. Zipping two different domain types
   (`[V]T` with `[E]T`) stays a type error. (R04b.1.3)
4. **Ends and relative reads are named.**
   - `w.first` and `w.last` are the first and last elements (`?T`).
   - `w.nth(k)` is the element `k` steps from the start. It is 0-based,
     O(1), and traps past the end.
   - `w.keys.nth(k)` is the k-th key, because a range is an indexed
     sequence too. (R04b.1.4)
5. **An array can cover any run of keys.** `[r]T` for a range `r` of `K` is
   indexed by `K` and valid on `r`, so `[s.keys]0` is a table shaped like
   `s`. A declared domain's array covers the whole domain, as before.
   `[0..=n]Int` is the case we already have, where `K` is `Int`. (R04b.1.5)
6. **Materializing keeps keys when it can.**
   - `.collect()` of an indexed view gives `[w.keys]T`.
   - `.collect()` of a walk-only sequence gives a `List` keyed from 0.
   - `List(w)` and `Str(w)` always count from 0. They are the explicit way
     to renumber. (R04b.1.6)
7. **A sequence can get a domain of its own.**
   - `w.rebase` is an O(1) view keyed by a fresh domain `0..w.len` that
     belongs to `w` alone (the `[domain]T` package of 2.14). Binding it with
     `let t = w.rebase` names the domain `t.Idx`.
   - `t.Idx` has every domain power: `Int(i)`, `t.Idx(k)`, `t.Idx.wrap(x)`,
     `[t.Idx]U` tables. Its keys can't be mixed with the source's or with
     another rebase's.
   - `a.rekey(r)` is the inverse: the same cells re-keyed onto a range `r`
     of equal length, O(1), trapping on a length mismatch. Library code uses
     it to hand results back in the caller's keys. (R04b.1.7)

Diagnostics:
- **Lint: literal index into a slice** (`w[0]`, `w[1]`). Suggests `w.first`
  or `w.nth(1)`. (R04b.1.8)
- **Lint: 0-based counter used as a slice key** (`for k in 0..w.len: w[k]`).
  Suggests `w.items`, `w.nth(k)` or `w.rebase`. (R04b.1.9)
- **Trap: an out-of-keys read.** The message names the key range, e.g.
  "`w` has keys 3..7 (it is `s[3..7]`); use `w.nth(0)` for its first
  element". (R04b.1.10)
- **Compile error: a raw `Int` used to index a domain slice**, as before. (R04b.1.11)

## Contest code

**Divide and conquer on the maximum (B1).** The position found inside the
slice is a position of `a`, so it indexes `b` and splits the range directly.

```python
fn solve(r: Range<V>) -> Int =
    if r.len == 0: return 0
    let m = a[r].argmax()!               // m: V, valid on a and b
    solve(r.l..m.l) + solve(m.r..r.r) + b[m] * r.len
```

**Search in a suffix (B2).** `find` answers in `s`'s keys, so the token is
`s[l..j]` with no `+ l`.

```python
var l = 0
while l < s.len:
    let j = s[l..].find(',') ?? s.len    // a key of s
    words.push(s[l..j])
    l = j + 1
```

**A per-element result on a piece (E2).** The table is keyed like the
piece, so `i` from `s[l..]` reads it directly.

```python
let z = z_function(s[l..])               // [l..s.len]Int
let hits = s[l..].keys.count(i => z[i] >= k)   // suffixes starting with s[l..l + k]
```

**Parallel arrays over a range (B3).**

```python
for (i, x) in a[l..=r].items:            // i runs over l..=r
    best[i] = x * w[i]
```

**Consuming from the front (A7).** The keys of `rest` shrink, and the
elements never move.

```python
var rest = a[..]
while let x = rest.first:
    if x > budget: break
    budget -= x
    rest = rest.drop(1)
```

**Comparing pieces (A2, rule 3).** Pairing is by walk order, so the offsets
don't matter.

```python
if s[i..i + k] == s[j..j + k]: ...
let lcp = zip(s[i..], s[j..]).take_while((x, y) => x == y).count()
```

**Relative reads.**

```python
s[l..].nth(k)        // the k-th character from l; same as s[l + k]
w.first!, w.last!    // the ends of a non-empty piece
```

**Grids (F1).** A row keeps the column domain and a column keeps the row
domain.

```python
for (c, x) in grid[r].items: ...         // c: Col
let col = grid[.., c]                    // [Row]T view
```

**A 0-based algorithm on a piece (rule 7).** `rebase` gives a domain for
the piece, and the domain's own tools (`wrap`) do the arithmetic.

```python
let t = s[l..r].rebase                   // t: [t.Idx]Char, keys 0..r - l
let rotated = Str(t[t.Idx.wrap(Int(i) + k)] for i in t.keys)
```

## User helpers

**Preferred: generic over keys.** A parameter typed `[K]T` is a view type:
it accepts an array, a `List`, a `Str` or any slice of them (any
`Indexed<K, T>`; with `mut`, any `IndexedMut<K, T>`), and the helper
answers in the caller's keys. It walks `a.keys`, never all of `K`: on a
slice, `a[k]` outside `a.keys` traps in checked builds. Only an owned
`[D]T` promises every key of `D`. This is the same code as stdlib code, with no extra power.

```python
fn first_drop<K>(a: [K]Int) -> ?K =
    a.keys.drop(1).first(i => a[i] < a[i.prev!])

fn next_greater<K>(a: [K]Int) -> [K]?K =  // keyed like a
    var res = [a.keys]none
    var stack = List<K>()
    for i in a.keys:
        while let j = stack.last and a[j] < a[i]:
            res[j] = i
            stack.pop()
        stack.push(i)
    res
```

Both work unchanged on `a`, on `a[l..=r]` and on `a.rev`. Their results
index the source.

**A helper that takes a container.** `List<T>` and `Str` parameters accept
only that container, because the callee may grow it (`push`, `pop`,
`clear` need the concrete type). `fn f(xs: List<Int>)` called with
`s[3..7]` is a type error with two fixes:
- make `f` generic as above, using `first`, `nth` and `items`; (R04b.3.1)
- call `f(List(w))`, an O(len) copy keyed from 0. (R04b.3.2)

## Library code

Stubs keep `= todo` bodies. The bodies here show that "keys in, keys out"
costs a library author nothing.

**Position queries return keys.**

```python
/// O(n). A key of the largest element, the first on ties; `none` when empty.
pub fn argmax<K, T: Ord>(self: [K]T) -> ?K = self.keys.max(by: k => self[k])

/// O(n·|p|). The first key where `p` starts, valid on the source of a slice.
pub fn find<S: Indexed<K, Char>, K, P>(self: S, p: P) -> ?K = todo
```

**Per-element tables are keyed like the input.** The algorithm runs on
offsets through `nth`, and `rekey` hands the table back in the caller's
keys.

```python
/// O(n). `z[i]` is the longest common prefix of `s[i..]` and `s`, keyed by `s.keys`.
pub fn z_function<K, T: Eq>(s: [K]T) -> [K]Int =
    let n = s.len
    var z = [0..n]0
    var (l, r) = (0, 0)
    for i in 1..n:
        if i < r: z[i] = min(r - i, z[i - l])
        while i + z[i] < n and s.nth(z[i]) == s.nth(i + z[i]): z[i] += 1
        if i + z[i] > r: (l, r) = (i, i + z[i])
    z.rekey(s.keys)
```

**Prefix tables use the input's cuts,** so `a[l..r].scan(+)[c]` takes a cut
`c` of `a`:

```python
pub fn scan<K, T>(self: [K]T, f: (T, T) -> T, from: ?T = none) -> Prefix<K, T> = todo
```

**Getting a domain from what you were given (rule 7).**

| Need | Write | Gives |
|---|---|---|
| Work in the caller's keys | take `[K]T` | `K` itself; nothing to build (R04b.4.1) |
| A table over exactly `s`'s keys | `[s.keys]x` | `[K]T` valid on `s.keys` (R04b.4.2) |
| A private 0-based index space for `s` | `let t = s.rebase` | view over fresh `t.Idx` (R04b.4.3) |
| An owned list as a domain array | `let (domain I, b) = xs.with_domain()` | `[I]T` that owns its cells (R04b.4.4) |
| Results back in the caller's keys | `table.rekey(s.keys)` | O(1) re-key (R04b.4.5) |

## Spec changes this implies

| Stub | Today | Becomes |
|---|---|---|
| `find`, `rfind` | `-> ?Int` on `Str` | `-> ?K` on any `Indexed<K, Char>` (R04b.5.1) |
| `find_all`, `shift_and`, `KmpAutomaton.find_all` | `yields Int` | `yields K` (R04b.5.2) |
| `z_function`, `prefix_function`, `prefix_matches` | `-> List<Int>` | `-> [K]Int`, keyed by `s.keys` (R04b.5.3) |
| `min_rotation`, `max_rotation` | `-> Int` | `-> K`, the start (R04b.5.4) |
| `lyndon_factorization` | `-> List<Int>` | `-> List<K>`, the starts (R04b.5.5) |
| `Palindromes.longest_at(i)` | `i: Int` | `i: K` (R04b.5.6) |
| `period`, `borders`, `common_prefix_len`, `overlap` | `Int` lengths | unchanged (R04b.5.7) |
| `seq.cp` | `nth_element` | renamed `select(k)`, so `nth` means one thing (R04b.5.8) |
| `seq.cp` | none | `nth`, `rebase`, `rekey` added (R04b.5.9) |
| sequences doc §"Slices" | the hybrid | rule 1; the pending note goes (R04b.5.10) |

## Open

1. **Should `sorted()` of a slice keep its keys?** Rule 6 says yes:
   `a[l..r].sorted()[l]` is the minimum. The alternative is that `sorted`
   always returns a `List` from 0, as a sort produces a new order anyway.
   I lean towards keeping keys, for one rule.
2. **Is `rekey` worth a name?** The alternative is writing into
   `[s.keys]` through `s.keys.nth(i)`. That's correct, but it puts a key
   lookup in every hot-loop write.
3. **Should a `List<T>` parameter renumber automatically?** I say no: the
   parameter is lent the caller's slice with its keys, and that's what makes
   B and C safe. A helper that wants 0-based takes `List(w)`.
