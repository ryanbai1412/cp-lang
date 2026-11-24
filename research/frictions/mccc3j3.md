# Friction report: mccc3j3

## Dream

```cp
let s: [domain S]Letter = read()
let t: [domain T]Letter = read()
let (in_s, in_t) = (s.tally(), t.tally())
print(sum(in_s[c] * in_t[c] for c in Letter) + max(0, in_s.max() - in_s[t.values].min()))
```

## Letters read as `Char`

The strings are read as `[S]Char` and turned into `Letter` per character. Reading straight
into `[S]Letter` (a `'A'..='Z'` domain element per character) would type the alphabet at the
read.

## Counting letters by hand

`in_s` and `in_t` are filled by `for c in s: in_s[Letter(c)] += 1`. A `tally()` into a
`[Letter]Int` table would be one expression each.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). The max case
(|S| = |T| = 1e6) runs in 0.01 s. Corpus tests pass.
