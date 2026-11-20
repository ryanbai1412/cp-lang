# Blind critic 3: contest test of the one-page cheat sheet

I learned CP Lang from `docs/one-page-cheat-sheet.md` (as of `d342632`) and nothing else, then
wrote first drafts of eight hard corpus problems that had no port. Only after all the drafts
were on disk did I build them with `proto/cpc`, fix them until they built, and differential-test
each one against its original C++ submission. Drafts and final ports are in
[`proto/ports/blind_critic_3/`](../proto/ports/blind_critic_3/).

The short version: **the language on the sheet is pleasant and I got the algorithms right first
time on all eight, but the sheet and the prototype are two different dialects.** One draft of
eight built unchanged. The other seven failed on 17 distinct errors, and every one of them was
a sheet-vs-prototype mismatch or a hole in the sheet, not an algorithm bug. Once a draft built,
it passed the differential test on the first run in seven of eight cases (the eighth needed
the original's `1e9` cap for an unbounded answer).

## What was tested

Each line is: problem, topics, original submission, non-blank non-comment lines of the draft /
the final port / the original C++.

- `noip19p2`: tree DFS + bracket matching (the original uses a segment tree with a custom monoid over the root path). 5127828. 27 / 27 / 186.
- `coci21c2p5`: compression + range-add/max tree + two pointers + binary lifting. 4143748. 33 / 34 / 143.
- `aac5p5`: sweep line over y + two Fenwick trees over compressed x. 4240171. 47 / 67 / 155.
- `cco16p1`: DSU over paths and cycles + counting. 4583882. 19 / 21 / 105.
- `cheerio1s4`: implicit graph on next/previous occurrences + Dijkstra. 5339437. 34 / 35 / 93.
- `ioi09p6`: multi-source grid BFS + binary search on the answer. 5106126. 35 / 56 / 128.
- `cmt21r1xym`: sorting + counting + binomials mod p. 3997206. 30 / 36 / 104.
- `noip22p1`: grid run lengths + running sums mod p, multi-test. 5213639. 32 / 36 / 108.

All eight final ports agree with their originals on 300 random inputs each
(`python3 proto/ports/blind_critic_3/check.py`). The generators are small-biased (sizes up to
a few hundred); nothing here was run at max N, so there are no timing or memory claims.

A ninth problem, `btoi19p2` (grid + `Bitset<Col>` shifts), was drafted and ported, then dropped
because `scout_batch_20` landed a port of it while this run was in progress. Its two errors are
still in the log below because they are real cheat-sheet findings.

There is no strings + binary search problem in the set: I could not find an unported AC
submission in the corpus that combines the two. `ioi09p6` (character grid + binary search on
the answer) is the closest. So `z_func`, `kmp`, `SuffixArray` and `RollingHash` are untested
here.

## Cuts that felt great

**Optional steps and `if let` replace every bounds check.** In `cheerio1s4` the four
"previous/next occurrence" arrays are `[0..n]?Int`, the C++ `-1` sentinels disappear, and the
neighbour loop reads the way I think about it. This built and ran as drafted:

```
for v in [prv[u], nxt[u], prv2[u], nxt2[u]]:
    if let w = v: relax(w, d + abs(u - w))
```

**Nested functions that mutate outer `var`s.** `noip19p2` is a recursive DFS that pushes and
pops a shared stack and xors into a shared answer. No capture lists, no `mut` threading, and it
was the one draft that built and passed unchanged. This is the single biggest "it just works"
on the sheet.

```
fn dfs(u: V, acc: Int):
    var popped: ?V = none
    if s[u] == '(': open.push(u)
    else:
        popped = open.pop()
        if let o = popped:
            f[u] = 1
            if let p = par[o]: f[u] += f[p]
    ...
```

**`Mod<P>` as a type.** `noip22p1` and `cmt21r1xym` have zero `% MOD`; mixing `Z` with `Int`
(`ansc += sc * w`, `ansc * kc`) worked without thought.

**`x min= y`, `1e18` as an `Int`, comprehensions with filters** (`sum(1 for j in I if g[i, j] == '1')`),
`[D] i => expr` array builders, and `while let (d, u) = pq.pop():` all read well and worked.

**Explicit `mut` at call sites** (`solve(rows, mut ma, mut wa)`) cost nothing to write and made
the one out-parameter function obvious.

## Cuts that caused friction

**The sheet never says what `[0..n]T` is.** Every example indexes by a named `domain`. Under
pressure I wrote `[0..n]Int` everywhere because that is what a C++ hand does, and it mostly
worked with `Int` indices. Then two different surprises: two spellings of the same anonymous
range are different types, and switching to a named domain to fix that makes `Int` indices
illegal. In `cmt21r1xym` that was three rebuilds in a row:

```
fn solve(cnt: [I]Int, mut best: [0..=n]Int, mut ways: [0..=n]Z): ...
var ma: [0..=n]Int = 0
solve(rows, mut ma, mut wa)      // error: expected [range]Int, got [range]Int
```

**No `Deque` and no `(r, c).around()` in the prototype** turned the 35-line `ioi09p6` draft
into 56 lines: a hand-written `around` returning a list, and two `List` + head-index queues.
The sheet version is exactly what I wanted to write:

```
for (r, c) in p.around():
    if g[r, c] != 'T' and not seen[r, c] and safe((r, c), (d + 1) / kv): ...
```

Related: the sheet does not say whether a `(R, C)` tuple can index a grid (`bee[p]`,
`seen[start]`). I assumed yes. If the answer is no, `around()` loses half its value.

**Fenwick and lazy segment tree spellings do not match.** The sheet says
`LazySegTree<C, Int>(AddMax, a)` with `.apply(l..=r, d)` / `.fold(l..=r)`; the prototype has
`AddMax<C>()` with `.add(l, r, d)` / `.max()`. The sheet says `Fenwick<D, Int>(a)` with
`.sum(l..=r)`; I could not get a Fenwick out of `proto/cpc` quickly and hand-rolled one in
`aac5p5` (20 extra lines, the largest draft-to-final growth).

**`compress` returns three things on the sheet and two in the prototype**, and neither shape
is the one I wanted. With the 3-tuple I guessed that the middle array maps input position to
compressed value and wrote `id[2 * i]`, which depends on having pushed endpoints in a known
order. With the 2-tuple I wrote `coord.first_ge(iv[i].0)!` at every use. What I wanted is a
function from value to index.

**Whole-array reset has no spelling.** `seen = none` to clear a `[D]?Int` between passes and
`cnt = 0` inside a loop are the natural guesses; the sheet only shows `= 0` at declaration.

**Tuples that contain a `Mod` do not compile in the emitted C++** (`List<(Int, Z, Int)>`
generates an `operator<` that `Mod` lacks). That is a g++ template error wall rather than a CP
Lang diagnostic, the only one of the run.

**Things I hesitated on that the sheet does not answer:**

- Is `Int / Int` floor division? I wrote `x * (x - 1) / 2` and `(d + 1) / kv` and hoped.
- Does `ans min= dist[i]` work when the right side is `?Int` and `none` should be skipped? The sheet covers a `none` on the left only.
- Does `Bitset.shifted(k)` shift toward higher indices, and does it drop bits that leave `D`?
- Is `i + 1` on a plain `0..n` loop variable an `Int` or a `?I`? I used both in one file.
- `Map<K, List<T>>` append: I wrote `(by_y[h.y] ??= []).push(...)`, guessing `??=` is an expression.
- Does `read()` of `[R, C]Char` consume one row per line with no separators? (It does.)
- I wrote `q.push_back(...)` from C++ habit; the sheet says `Deque.push`. My error, but `push_front`/`pop_front` next to a bare `push` invites it.

## Exact first-draft error log

Errors are in the order hit, one build at a time, quoted from `python3 proto/cp.py build`.
"Fix" is what the final port does.

`noip19p2_draft.cp`: none. Built and passed 300/300 as written.

`cco16p1_draft.cp`

1. `error: line 2: unknown name read` on `let (n, m, k): (Int, Int, Int) = read()`, copied from the sheet's `let (n, m): (Int, Int) = read()`. Fix: three separate `let`s.
2. No error, wrong output: `print("{ans * k} {bad}")` printed the literal text `{ans * k} {bad}`. Fix: `print(ans * k, bad)`.

`noip22p1_draft.cp`

3. `error: line 4: unknown name read` (tuple read again, twice in this file).
4. Literal `{ansc * kc} {ansf * kf}` printed (interpolation again).

`cheerio1s4_draft.cp`

5. `error: line 2: unknown name read` (tuple read).
6. `error: line 14: \`none\` needs a known ?T type here` on `seen = none` for `var seen: [1..=m + 1]?Int`. Fix: a loop assigning each cell.
7. `error: line 20: expected MinHeap<(f0: Int, f1: Int)>, got List<None>` on `var pq: MinHeap<(Int, Int)> = []`. Fix: `MinHeap<(Int, Int)>()`. The sheet shows `= []` for `List` and no constructor for heaps.
8. `error: line 28: MinHeap<(f0: Int, f1: Int)> has no \`.is_empty\``. The sheet says every container has it. Fix: `while let (d, u) = pq.pop():`, which is better anyway.
9. `error: line 34: expected Int, got ?Int` on `ans min= dist[i]`. Fix: `if let d = dist[i]: ans min= d`.

`coci21c2p5_draft.cp`

10. `error: line 9: unknown name compress` on `let (C, id, _) = compress(all)` (the prototype only knows the 2-tuple form). Fix: `let (C, coord) = compress(all)` plus `coord.first_ge(x)!`.
11. `LazySegTree<C, Int>(AddMax, [C] c => 0)`, `.apply(l..=r, d)`, `.fold(C.first..=C.last)` do not exist. Fix: `AddMax<C>()`, `.add(l, r, d)`, `.max()`.

`cmt21r1xym_draft.cp`

12. `error: line 7: unknown type Comb`. Fix: a Pascal table `[0..=n, 0..=n]Z`.
13. `error: line 13: prefix takes a 1-D or 2-D array of Int or Bool, got List<Int>`: `sorted(cnt, by: ...)` of a `[I]Int` returns a `List`, so `prefix` and `pre.sum(I.first..=I.from_idx(i - 1)!)` were dead. Fix: a running sum. The draft line was ugly enough that this was a mercy.
14. g++: `error: no match for 'operator<' (operand types are 'const Mod<998244353>' and 'const Mod<998244353>')` from `List<(Int, Z, Int)>`. Fix: out-parameter arrays.
15. `error: line 18: Int index \`i\` into array over K; convert with K(x) or test \`x in K\`` after introducing `domain K = 0..=n`.
16. `error: line 30: expected [range]Int, got [range]Int` after going back to `[0..=n]Int` in both the signature and the caller. Fix: named `K` and `K(i)` at each index.

`aac5p5_draft.cp`

17. `error: line 17:29: expected an expression (at '=')` on `(by_y[h.y] ??= []).push((cx(h.x1), cx(h.x2)))`. Fix: compress y and group in a `[Y]List<(Int, Int)>`.
18. Not built further as drafted: `sort(mut xs)` + `dedup(mut xs)` + `xs.first_ge(x)!` as a hand compression, `Fenwick<0..K, Int>()` with `.sum(l..=r)`, a nested `fn upd` declared inside a `for` body, and `for (y, s) in by_y.items` were all replaced when the port moved to `compress` and a hand-rolled Fenwick. The rewrite then hit error 16 again (`[0..=K]Int` parameter vs global).

`ioi09p6_draft.cp`

19. `error: line 21:47: expected an expression (at 'let')` on `return if let b = bee[p]: b > t else: true`. The sheet has inline `if` and `if let` but does not say they do not combine.
20. `Deque`, `p.around()`, tuple-indexed grids, `first(... for ...)` and `bsearch_last(0..=hi, t => check(t))` are all absent from the prototype. Fix: see the friction section.
21. Wrong answer after it built: `4` vs `1000000000` when the bees can never reach the bear. Mine, not the language's: my search range was `n * n`, the original's is `1e9`.

`btoi19p2_draft.cp` (dropped from the set, see above)

22. `error: line 9: unknown type Bitset`. Fix was a `[Row, Col]Bool` grid, 64 times slower.
23. `error: line 11: cannot iterate Str` on `for ch in moves` with `let moves: Str = read()`. Fix: `[0..m]Char`.

## Top 5 fixes before Architect Wave 2

**1. Make the sheet's first page compile on the prototype, and gate it in CI.** Tuple read
(`let (n, m): (Int, Int) = read()`) and string interpolation (`print("{a} {b}")`) are the first
two things a contestant types, and they were errors 1 to 5. Worse, interpolation fails silently
by printing the braces. Extract every fenced block from the sheet into `proto/tests/` and build
it; anything that cannot be built yet gets a visible "(planned)" mark on the sheet. Today a
reader cannot tell which third of the sheet is real.

**2. Decide what `[0..n]T` means and say it in one line.** Either anonymous ranges are
structural (same bounds, same type, `Int` indices) or they are banned in type position. The
current behaviour, where it works until the array crosses a function boundary and then fails
with `expected [range]Int, got [range]Int`, is the worst of both. My preference: structural,
with a sentence on the sheet such as "`[0..n]T` is a plain 0-based array indexed by `Int`;
name a `domain` when you want index safety". At minimum fix the diagnostic to print the bounds.

**3. One spelling for range structures, and ship the ones the sheet lists.** Pick
`.sum(l..=r)` / `.apply(l..=r, d)` / `.fold(l..=r)` (sheet) or `.sum(l, r)` / `.add(l, r, d)` /
`.max()` (prototype) and use it for `Fenwick`, `SegTree` and `LazySegTree` alike. Then make
`Fenwick<D, Int>()`, `Deque<T>`, `Bitset<D>` and `Comb<P>(n)` exist in `proto/cpc`: those four
missing tools account for most of the 41 lines the final ports grew over the drafts.

**4. Give `compress` a value-to-index lookup.** Both shapes forced a detour. Something like

```
let xs = compress(all)            // xs.D is the domain, xs[v] the value at v
st.apply(xs.at(l)..=xs.at(r), 1)  // xs.at(value) -> D, traps if absent
```

removes the positional `id[2 * i]` guess and the repeated `coord.first_ge(x)!`.

**5. Close the small holes the sheet leaves open, one line each.** In order of how much time
they cost me:

- Container construction: say `var h: MinHeap<T> = []` works for every container, or show `MinHeap<T>()`. And either give heaps `.is_empty` or stop saying every container has it.
- Whole-array reset: `a.fill(x)` or allow `a = x` after declaration.
- `x min= y` with an optional right side (skip `none`), since `dist` arrays are `?Int` by design.
- `if let` as an expression, or a note that it is statement-only.
- Whether a `(R, C)` tuple indexes a `[R, C]T` grid.
- That `/` is floor (Euclidean) division.
- `for ch in s` over a `Str`.
- Tuples containing `Mod` must compile; if ordering is the issue, derive `<` only when it is used.
