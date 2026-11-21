# 29. Strings

**Layer:** library. `Str` is `[len]Byte` with its own `Index` impl (integer positions, each string its own `Cut`); helpers are library. Status: [proposed].

## 29.1 Strings

- `Str` is bytes. **It is indexed by the integer range `0..s.len`** [r7 12], so positions are plain
  integers. `s[i .. j]` is a slice and prints as text.
- `s[i]` is a `Byte`, with literals `'a'`. The stdlib's `Letter = labels('a'..='z')` converts with `Letter(c)`
  (checked) or `Letter.try(c) -> ?Letter`, like any conversion [r12 R4, p1 S1], so `[Letter]Int` replaces `[26]Int` plus `c - 'a'` [r11 N30].
- A read type may bound the length: `Str(len: 1..=1e4)`, checked like any range. A dense `@memo` over an
  unbounded `Str` length is a compile error naming this fix [r11 N27].
- `s.prefix_function() -> [s.len]0..s.len`, `s.z_function()`, `s.borders()` (proper non-empty border lengths,
  longest first) [r16 R7].
- Derived index spaces stay nominal: `SuffixArray(s)` returns `order: [Rank]0..s.len` and
  `lcp: [Rank]0..=s.len` over a fresh `Rank`, so a rank can't index the string.
- `s.digits()` gives `[s.len]0..=9`; `s.windows(m)` gives the nonempty length-m slices.
- Hashing: `Hash2` is `(Mint(mod: P1), Mint(mod: P2))` with a random base chosen on the judge, seed printed
  to stderr unless the judge profile counts stderr [r11 N34]. Local runs use a fixed seed; stress treats the seed as a generator input; `with hash(seed: s)`
  pins it [r7 13].

- **Strings:** `Str.filled(len, b)`, `Str(bytes)`, `Letter.upper`.
