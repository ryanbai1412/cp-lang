# Friction Report: cco04p2

The v0.2 port parsed `x-y` digit by digit into a `List<Int>`, which no longer type-checks
(`partition_point` was removed). It now reads `words: [s]Str` and splits each one with
`split_once('-')` and `parse_int`. The statement's `s <= 1000` is a type and the package gains
tests.toml.

## A dash-joined pair is read as a string and parsed by hand
- **Where:** `fn score(word: Str) -> Score`, with two `!` unwraps.
- **Ideal:** a typed read with a separator, e.g. `let scores: [s](Int, '-', Int) = read()`, or
  `Str.parse<(Int, Int)>(sep: '-')` (`parse<T>` is a `todo` in std today). Backlog item S8 covers
  the same gap in gen.py.

## Dream
```cp
@!multitest
let s: 1..=1000 = read()
let scores = [(min(x, y), max(x, y)) for (x, y) in (read(): [s](Int, '-', Int))].sorted()
var games: List<List<(Int, Int)>> = []
for (a, b) in scores.rev:
    match games.first_index(game => game.last!.1 >= b):
        Some(g): games[g].push((a, b))
        None: games.push([(a, b)])
print(games.len)
for game in games: print(["{a}-{b}" for (a, b) in game.rev])
```

- Tokens: port 230, original 586
- Max input (`gen.py --max`): port 0.03 s, 3836 KB, original 0.05 s, 3584 KB
