# Friction report: noip20p2

There is no dream.cp; the ideal is below. The port fixes w = |AB|:
- `z_function(s)` gives the number of repeats;
- `odd_prefixes` gives F of every prefix and suffix (a `Bitset<Letter>` toggled per letter);
- `at_most[d]` grows by one prefix per w.

## Dream

```cp
@multitest
let s = read<List<Letter>>()
let before = s.scan(Bitset<Letter>(), (b, c) => b.toggled(c)).map(.len)
let after = s.scan_rev(Bitset<Letter>(), (b, c) => b.toggled(c)).map(.len)
```

## Suffix F needs two reversals and a `List` rebuild
- **Where:** lines 31-32: `let letters = [Letter(c) for c in s]` and `let after = List(odd_prefixes(letters.rev).rev)`.
- **What felt wrong:** "F of every suffix" is spelled as reverse, scan, reverse, and copy. The `Str` is also converted letter by letter into a `List<Letter>` that a typed read could produce.
- **Goal it hurts:** express.
- **Ideal code:** the dream's `scan_rev` and `read<List<Letter>>()`.
- **General fix:** a std `scan_rev` (suffix scan) on lists. A `List<Letter>` read for a lowercase token is a separate question for Core. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 232, accepted C++ 955 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.50 s / 50 MB, accepted C++ 0.42 s / 47 MB, with the same output.
- **Emitted C++:** 21227 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
