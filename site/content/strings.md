`Char` is one ASCII byte. `Str` is a flat, value-semantic sequence of those bytes indexed by `0..s.len`. There is no UTF-8 decoding, variable-width character indexing, Unicode normalization, or hidden code-point iterator behind a subscript.

> **Deliberate scope.** Non-ASCII text is outside the `Char`/`Str` contract. Character and string literals must be ASCII. Arbitrary binary bytes belong in an explicit byte container rather than a string that silently changes encoding.

## Value-semantic, `0..len`-indexed `Str`

In CP Lang 0.2.2, default `Str` is a flat `0..s.len`-indexed ASCII byte sequence with O(1) indexing and value semantics. Slicing currently materializes a new string rather than a constant-time view:

```cp CP Lang 0.2.2
let s: Str = read()
if not s.is_empty:
    let ch = s[0]               // Char; digit: ch - '0', letter: ch - 'a'
    let sub = s[..min(3, s.len)]
    print(ch, sub)

var out = ""
out.push('a')
for _ in 0..3: out += "bc"      // mutable byte buffer when `var`

if s.len >= 2 and s[..2] == "ab":
    print(out)
```

Because `Str` has value semantics, `var t = s` moves or copies cleanly without hidden aliasing; call `s.copy()` when you want an independent mutable copy while keeping `s`. You can run `Str` indexing, slicing, concatenation, and ASCII arithmetic directly in the offline [playground](playground.html) (pick **ASCII string scan**).

## Common string helpers

The current toolchain supports these operations directly:

- Indexing and slices: `s[i]`, `s[l..r]`, `s[..k]`, `s[k..]`, `s.len`, `s.is_empty`.
- Building: `var out = ""`, `out.push(c)`, and `out += t`. String repetition with `*` is not yet implemented; append in a loop.
- Prefix matching: guard the length, then compare `s[..prefix.len] == prefix`. For input, `read()` parses whitespace-delimited typed tokens.
- Character arithmetic: `c - '0'`, `c - 'a'`, `'A' + (c - 'a')`, and character ranges `'a'..='z'`.

## Opt-in nominal character domains

When you are writing a two-string DP (such as LCS or edit distance) and want the compiler to forbid mixing an index into string `a` with an index into string `b`, opt into a nominal character domain `[domain A]Char`:

```cp CP Lang 0.2.2
let a: [domain A]Char = read()
let b: [domain B]Char = read()

var dp: [A.bounds, B.bounds]Int = 0
for i in A:
    for j in B:
        dp[i.r, j.r] = if a[i] == b[j]: dp[i.l, j.l] + 1 else: max(dp[i.l, j.r], dp[i.r, j.l])
print(dp[A.last.r, B.last.r])
```

Swapping an `A` boundary with a `B` boundary is a compile error, and `.l`/`.r` carry the empty-prefix boundary cleanly. When the length is already read into a domain, the same `[S]Char` form runs directly in both `cpl` and the browser WebAssembly playground:

```example ascii
```

For input `5` and `abaca`, the result is `3`. Each subscript touches one contiguous byte in O(1).

## String algorithms in `std`

The standard library provides `z_function(s)`, `prefix_function(s)`, `palindromes(s)`, `SuffixArray(s)` (`.sa`, `.lcp`), and double-salted `RollingHash(s)` (`.get(l..r)`), all operating directly on flat ASCII bytes. For a real multi-DP string port using `0..n`-indexed `Str`, see the [Cheerio '1 S5 showcase](index.html#showcase-cheerio1s5).

## Prefix, suffix, and search helpers

`Str` has `.starts_with(p)`, `.ends_with(p)` and `.find(p) -> ?Int` (the first index where `p` starts), and `Char` has `.lower` and `.upper` for ASCII letters and `.digit` for `'0'..='9'`. `s.find_all(p)` lists the start of every match, overlaps included. All of them are in the prelude.

```cp
let s: Str = "hello"
print(s.starts_with("he"), s.ends_with("lo"), s.find("ll"))   // YES YES 2
print('a'.upper, 'Q'.lower)                                  // A q
```
