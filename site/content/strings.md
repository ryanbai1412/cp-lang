`Char` is one ASCII byte. `Str` is a flat array of those bytes. There is no UTF-8 decoding, variable-width character indexing, Unicode normalization, or separate code-point iterator hiding underneath a subscript.

> **Deliberate scope.** Non-ASCII text is outside the `Char`/`Str` contract. A character or string literal must be ASCII, and input for those types must satisfy that constraint. Arbitrary binary data belongs in an explicit byte container, not in a string that silently changes encoding.

## The familiar array model

When the input gives a length, use a domain-indexed character array. This example counts occurrences of a character in a single ASCII token.

```example ascii
```

For input `5 abaca`, the result is `3`. Each access selects exactly one byte. The stored character array is contiguous and supports O(1) indexing; there is no walk to the nth UTF-8 code point.

## A string carries its own index space

In the approved unified design, `Str` is an alias for `[domain]Char`: an owned array packaged with a fresh index type. Opening the package as `s` exposes that type as `s.Idx`. Two independent strings have different position types, even when they have the same length.

The ASCII restriction is implemented in the prototype. The `[domain]Char` package and `s.Idx` interface are still design-only.

```cp CP Lang · design
let s: Str = read()
let t: Str = read()

for i in s.Idx:
    print(s[i])
```

`t[s.Idx.first!]` is not a shortcut for the first character of `t`; it mixes two index spaces. The `!` would also require that `s` is nonempty. Use positions belonging to the string you actually want.

## Slices and boundaries are shared tools

`s[l..=r]` lends a character slice just as `a[l..=r]` lends an integer slice. It does not allocate a substring. `.copy()` explicitly makes independent storage. Whole-slice operations and iteration can prove membership; an arbitrary parent position needs a window check.

For a two-string DP, put the empty prefix on a boundary rather than inventing an extra numeric row and column:

```cp CP Lang · design fragment
var dp: [s.Idx.bounds, t.Idx.bounds]Int = 0
for i in s.Idx:
    for j in t.Idx:
        dp[i.r, j.r] = if s[i] == t[j]: dp[i.l, j.l] + 1
                       else: max(dp[i.l, j.r], dp[i.r, j.l])
```

This is the LCS recurrence expressed over the two strings' own indices. Swapping an `s` boundary with a `t` boundary is a type error. `.l` and `.r` have the same meaning here as they do in [integer prefix sums](indices.html#prefix-sums-without-shifts).

## Owning collections of strings

`List<Str>` can hold strings of different lengths because each element owns its own package. A string opened while iterating is lent for that iteration. Keep `.copy()` if the result needs independent ownership, rather than storing a borrowed window.

String equality, ordering, and hashing compare contents; sharing an index type is not required to compare two strings. Build growing text with `List<Char>`, then freeze/package it as a `Str`. The library is responsible for these operations, not a second string-only type system.

## String algorithms, not string magic

The standard-library plan includes prefix functions, Z-functions, hashing, and suffix arrays. They should accept the same bounded character arrays as ordinary sequence code. The [Searching for Strings port](https://github.com/ryanbai1412/cp-lang/blob/main/proto/ports/ccc20s3.cp) is a concrete prototype example: character domains for letter counts, a moving window, and substring hashes.
