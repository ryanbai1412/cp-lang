# Friction backlog, batch 10: strings

A Stage-1 scout pass over ten string-algorithm submissions: KMP, the
Z-function, suffix arrays, rolling hashes and a trie. The ports are in
[`proto/ports/scout_batch_10/`](../proto/ports/scout_batch_10/): ctsc06p1,
btoi19p5, noip20p2, saspeed, noi15p5, nccc10s3, noi16p1, noi17p2,
year2018p7 and sac22cc3p5. Items are numbered `T1`... so they do not
collide with `F` (batch 1), `G` (3), `B` (2 and 4), `K` (5), `S` (6), `D`
(7) or `H` (8). An item an earlier batch already has is named with its
number and only what is new is added.

What was checked: all ten ports run on the reference interpreter
(`proto/cp.py lib-run`, at commit `91fb701`) and agree with their
originals on 60 small random inputs each
(`python3 proto/ports/scout_batch_10/check.py`). So every "today" snippet
below is a line a port really contains, and every error quoted came from a
small probe on that interpreter. The "wanted" snippets do not run. Nothing
was measured at full input size: the interpreter is too slow for that, and
T6 and T8 name the two ports that would be too slow even compiled. A count
such as "3 of 10" is the number of the ten ports that contain the awkward
line.

No port is a line-by-line translation. Four changed algorithm because the
library had a better one to hand: noip20p2 and btoi19p5 dropped the
original's rolling hash for `z_function` and `prefix_function`, ctsc06p1
its recurrence for a sum over borders, and sac22cc3p5 a hand-built hash
table for a trie.

The corpus has no accepted, unported submission that uses Aho-Corasick or
Manacher, so the batch has neither; T10 covers what a trie in the library
would need for the first.

## Suggested order for a Stage-2 sprint

1. T1: move `hashing.cp` into `stdlib/`. It is written and tested, three
   ports use it, and six originals each carry their own copy of it.
2. T3: let `std.strings` take a string. Six ports convert by hand.
3. T4 and T5: the two functions that would have made the suffix-array and
   KMP ports read as their comments do.
4. T6 and T8: speed, with no change to any port.
5. The rest, each seen once.

## T1. A hash that is a value (3 of 10: noi16p1, noi17p2, year2018p7)

`StrHash(s)` exists, but inside `proto/cpc`, and `hash.of(range)` is a bare
`Int`. So a port that runs with the library cannot use it, and a hash
cannot be joined to another hash, because the number has forgotten its
length. noi17p2 needs exactly that: the blocks that cross a join are a
hash of worms up to `u` followed by a hash of worms from `v`. Six of the
ten originals carry their own hasher (noi16p1, noi17p2, year2018p7,
noip20p2, btoi19p5, nccc10s3); noi17p2's has `operator+` and `sub_pref`,
btoi19p5's `merge` and `rot`.

Wanted, and what the ports contain today because the module was written
for this batch as [`hashing.cp`](../proto/ports/scout_batch_10/hashing.cp)
(102 lines, userland, two tests):

```cp
use scout.hashing.{Hash, StrHash}

let hash = StrHash.new(s)
hash.of(i..i + k)                                   // a Hash: a length and two residues
hash.lcp(q, q + w)                                  // letters shared from two places; noi16p1
block = Hash.one(size[w]) + block                   // joined end to end; noi17p2
var blocks = Map<Hash, Int>()                       // a key like any other
```

`Hash` is a struct with `+` (join), `after` (cut a prefix off), `one` and
`empty`; `StrHash` is a list of prefix hashes with `of`, `lcp` and `lcs`.
Nothing in it is special to strings or to the compiler: `Mod<P>`, a list
and `impl Add`. Proposal: move it to `stdlib/hashing.cp`, and retire the
builtin `StrHash` in `proto/cpc` in favour of it, so there is one. G7's
`Hash2D` (batch 3) should return the same `Hash`.

## T2. `Mod<P>` stops at 3.03e9 (hashing.cp)

One residue modulo 2^61 - 1 is the usual safe hash. `Mod<P>` needs `P * P`
to fit an `Int`, so `hashing.cp` keeps two residues modulo two primes near
1e9, and every `Hash` operation is written twice:

```cp
Hash(x.len + y.len, x.a * pow_a[y.len] + y.a, x.b * pow_b[y.len] + y.b)
```

`std.math` already has `mul_mod(a, b, m)` for a large modulus. Proposal:
`impl Mul for Mod<P>` uses it when `P` is too large for the direct product
(a branch on a constant, so free), which makes `Mod<2305843009213693951>`
legal and `Hash` one residue. This is the wide-multiply intrinsic doing
its one job; no new construct.

## T3. A string is not a `List<T>` (6 of 10)

Every function in `std.strings` takes `List<T>`, and a string is read as a
`Str` or a `[I]Int`. saspeed, noi15p5, nccc10s3, noip20p2 and btoi19p5
convert a `Str`, ctsc06p1 an array:

```cp
let z = z_function([c for c in s])                       // noip20p2
let border = prefix_function([name[i] for i in I])       // ctsc06p1
```

*Checked:* `name.to_list` is "has no field or method `to_list`". The
interpreter does not check argument types, so `z_function(s)` happens to
run there; the signature says it should not. Wanted: `z_function(s)` for
any of them. Proposal: the functions take anything with a length and an
`Int`-indexed read, one trait (`Seq<T>`: `len` and `at(i)`), implemented
by `List<T>`, `Str`, `[D]T` and a slice. That is the same reading of "a
string is `[domain S]Char`" that G6 (batch 3) asks for on the way out.

## T4. A suffix array that knows what it sorts (3 of 10: saspeed, noi15p5, nccc10s3; batch 3 G6)

G6 asked for typed positions. Two ports here add what the type should be
able to do. nccc10s3 wants the suffixes that start with a pattern, and
writes a binary search and two comparisons of cut suffixes:

```cp
let below = leading(suffix => suffix[0..min(t.len, suffix.len)] < t)
let upto = leading(suffix => suffix[0..min(t.len, suffix.len)] <= t)
```

noi15p5 indexes scores through the order, with positions as raw numbers:

```cp
var lo: [Glass]Int = [Glass] j => score[Glass.nth(sa[j.to_idx])]
```

and noi16p1 wants the letters two places share, which a suffix array with
a sparse table over its LCP array answers in O(1) where the hash takes
O(log n). Wanted:

```cp
let sorted = SuffixArray(s)
sorted.matches(t)             // the range of ranks whose suffix starts with t
sorted.at(rank)               // a position of s, typed
sorted.lcp                    // between neighbours in sorted order
sorted.common(i, j)           // letters shared by the suffixes at i and j
```

Proposal: a userland struct in `std.strings` over `suffix_array`,
`lcp_array` and `SparseTable`, generic like the functions (T3).

## T5. How much of a pattern ends here (btoi19p5; ctsc06p1's original)

KMP's inner loop answers "the longest prefix of the pattern that ends at
this place in the text". `find_all` has the loop but only yields full
matches, so btoi19p5 gets it from `prefix_function` with a separator:

```cp
// '#' is in neither string, so no border crosses it.
let border = prefix_function(pat + ['#'] + text)
[0] + border[pat.len + 1..border.len]
```

This needs an item outside the alphabet, which a `List<Int>` does not
have, and it is off by `pat.len + 1` on the first try. Proposal:
`overlaps(pat, text) -> List<Int>` in `std.strings`, entry `j` for
`text[..j]`; `find_all` becomes the places where it equals `pat.len`.

## T6. `suffix_array` is O(n log^2 n) (saspeed)

saspeed has 10^6 letters and the original is 90 lines of SA-IS to meet it.
The port is one call, and right, and too slow there. Proposal: SA-IS
behind the same function, in `stdlib/`; no port changes. Not measured.

## T7. A union-find with something on each set (noi15p5)

`Dsu<V>` knows the size of a set. noi15p5 also needs its lowest and
highest score, keeps them in two arrays at the roots, and has to ask for
the roots before the union and again after it, because `unite` returns a
`Bool`:

```cp
let (a, b) = (groups.find(Glass.nth(j)), groups.find(Glass.nth(j + 1)))
...
groups.unite(a, b)
let root = groups.find(a)
(lo[root], hi[root]) = (min(lo[a], lo[b]), max(hi[a], hi[b]))
```

The original's `DSU` has the same shape with a `rep` array. Wanted:

```cp
var groups = Dsu<Glass, Span>.of(Semigroup(hull), j => Span(score[j], score[j]))
groups.unite(a, b)          // folds the two values
groups.value(a)
```

Proposal: an optional `Semigroup<T>` on `Dsu`, as `Fenwick` takes a
`Group`. D6 (batch 7) asks for a `Group` on the edges of the same
structure; they are two different things and both are small.

## T8. The k-th smallest in a range (nccc10s3)

The k-th occurrence of a pattern is the k-th smallest position among a
run of the suffix array. The port, like the original, sorts the run:

```cp
print(sorted(order[below..upto])[k - 1] + 1)
```

which is why the original special-cases a string of one letter. Proposal:
a static range k-th structure in `stdlib/` (a wavelet matrix, or a merge
sort tree), `RangeKth(order).kth(below..upto, k)`. D1 (batch 7) asked for
`kth` on a changing set; this is the fixed-array half.

## T9. Every border of a string (ctsc06p1)

The borders of a string are a chain through the failure function:

```cp
var (wait, b) = (0, n)
while b > 0:
    wait = ring.add(wait, ring.pow(m, b))
    b = border[b - 1]
```

The `b - 1` is the usual slip. Wanted: `for b in borders(name)`, the
string itself first, then each shorter border. `std.strings` has `period`
(the first step of this chain) and nothing for the rest. Proposal:
`borders(s) -> yields Int`, with `period` defined from it.

## T10. A trie (sac22cc3p5; batch 7 D4)

D4 said nodes in a growing list are named by `Int`. The trie here is that
again, with a second thing to get wrong, the new node's number:

```cp
if let next = trie[at].kid[letter(c)]: at = next
else:
    trie.push(Node([Letter] c => none, false, 0))
    trie[at].kid[letter(c)] = some(trie.len - 1)
    at = trie.len - 1
```

Wanted:

```cp
var words = Trie<Letter>()
let node = words.insert(word)         // a Node, not an Int
if let next = words.step(at, c): ...
```

Proposal: `Trie<A>` in `std.strings` over any alphabet domain `A`, on top
of whatever D4 settles for typed nodes in a growing pool. A table from
`Node` to anything (`[Node]T` once the trie is built) carries the "word",
"counted" and later the failure links of Aho-Corasick, which is then
`words.links()` in userland and not a second structure.

## T11. Counting in a map (noi17p2; batch 1 F18)

```cp
blocks[block] = (blocks.get(block) ?? 0) + by
```

F18 is resolved in the language: `m[k] += by` starts a missing key from 0.
*Checked:* on the interpreter, with `var k = Map<Str, Int>()`,
`k["a"] += 1` ends in a Python `KeyError`. `stdlib/steps.cp` writes the
long form twice for the same reason. This is an interpreter gap, listed
here because the line is the busiest one in the port.

## T12. Range additions read back at every place (noi16p1; batch 1 F19)

Squares start on a run of places, so the port adds on ranges and reads
every place afterwards:

```cp
starts[first] += 1
starts[last + 1] -= 1
...
let (begun, ended) = (running_sum(starts), running_sum(ends))
```

`std.steps` (from F19) has `add(range, x)` and reads back runs; here the
places are dense and wanted as a list. Proposal: `Steps.values(clip) ->
List<Int>` next to `runs`, so the port is `starts.add(first..=last, 1)`
and `starts.values(0..n)`.

## Seen again from earlier lists

- **Binary search written out (2 of 10, and twice in `hashing.cp`; F10,
  B1, D8).** nccc10s3's `leading`, year2018p7's search on the answer, and
  `lcp` and `lcs`. All four are `partition_point` over a range of `Int`s.
- **Small to large (year2018p7; F17, B6).** `swap(mut all, mut branch)`
  on two locals works and reads well. What is new: the merge also wants to
  know whether anything was already there, so `all.add(x)` returning
  whether `x` was new would fold two lines into one.
- **`unite` after `find` (noi15p5).** See T7.

## Smaller notes

- **Case (sac22cc3p5).** `if c.is_upper: Letter.nth(c - 'A') else:
  Letter(c)`. `c.lower` is not there. With one-byte characters it is a
  one-line prelude function.
- **A number in four digits (ctsc06p1).** The port builds `0676` from a
  `digit` helper. `"{wait:04}"` prints `676` on the interpreter: the
  width is read and ignored.

## Gaps in the reference interpreter

Not language friction: the language has the construct and `proto/cpi/`
does not.

- **`if let` as a value.** `best = if let old = best: max(old, corner)
  else: corner` is "expected an expression (at 'let')"; noi15p5 writes
  `max(best ?? corner, corner)`. `best max= corner` on a `?Int` is still
  the `TypeError` batch 2 reported.
- **An open slice.** `s[cut..]` is "cannot build a range from 1 to none",
  as in batch 7; btoi19p5 and nccc10s3 write `s[cut..n]`.
- **`m[k] += x` on a missing key.** See T11.
- **A tuple by number.** `t[0]` is "cannot index (2, 1, 2)"; `t.0` works.
- **Argument types are not checked.** See T3.

## Already good

- `z_function` and `prefix_function` turned two hash solutions into exact
  ones: noip20p2 is "how far does s agree with itself shifted by w", which
  is `z[w]`, and btoi19p5 is 24 lines against 204.
- `suffix_array` and `lcp_array` with `Dsu<Glass>` made noi15p5 30 lines
  against 285, with no index arithmetic beyond `j + 1`.
- A struct as a `Set` element and a `Map` key (`Set<Hash>`,
  `Map<Hash, Int>`) worked with no `impl`, and `impl Add for Hash` reads
  as the mathematics does.
- A library next to the ports (`lib.toml`, `use scout.hashing`) worked
  first time, with its tests under `cp.py test`.
- `[Worm]?Worm` for the queue links in noi17p2 and `if let w = at` for the
  walk: no sentinel worm, no `-1`.
