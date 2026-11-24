# Friction report: mccc5j4

## Dream

```cp
let s: [domain P](Letter | '*') = read()
var left = goal - s.letters.sum(value)
print(if wild <= left <= 26 * wild: s.fill_stars(greedy) else: "Impossible")
```

## Letter values by character arithmetic

`value(c) = c - 'a' + 1` and `'a' + (d - 1)` convert by hand between a `Char` and its 1-based
alphabet rank. A `'a'..='z'` letter domain with `.ord` and `Letter.at(i)` would carry the
bounds.

## Mutating the string in place

The greedy fill writes `s[p]` while walking `P` with a `var wild` and `var left`. A
stateful map (`scan`) building the filled string would remove the two `var`s.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). The max case
(|S| = 1e5) runs in 0.01 s. Corpus tests pass.
