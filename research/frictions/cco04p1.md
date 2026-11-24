# Friction Report: cco04p1

The v0.2 port indexed `[0..256]` arrays with `ch - 'a' + 97` and wrapped each word in a function.
It now keys `score` and `have` by `domain Letter = 'a'..='z'`, reads the tiles as one typed list,
and checks a word with `all(...)` and `sum(...)`. Adds gen.py and tests.toml.

## A character domain can't be read
- **Where:** `tiles: [k](letter: Char, ...)` then `Letter(t.letter)`, and each word read as `Str`
  then mapped with `Letter(c)`.
- **What felt wrong:** `let c: Letter = read()` fails at run time with "expected a number", so the
  domain is spelled in two places. A word also can't be read as `[domain W]Letter`, since its
  length isn't given.
- **Ideal:** reading a `'a'..='z'` domain reads one character, and `read<List<Letter>>()` reads a
  token as its letters.

## Dream
```cp
domain Letter = 'a'..='z'
let k: 0..=26 = read()
let tiles: [k](letter: Letter, score: 0..=26, count: 0..=7) = read()
let score = [Letter] c => tiles.find(t => t.letter == c)?.score ?? 0
let have = Counter(t.letter for t in tiles for _ in 0..t.count)
let n: 0..=1e5 = read()
let words: [n]List<Letter> = read()
print(max(0, max(sum(score[c] for c in w) for w in words if Counter(w) <= have)))
```

- Tokens: port 207, original 406
- Max input (`gen.py --max`): port 0.00 s, 3496 KB, original 0.00 s, 3456 KB
