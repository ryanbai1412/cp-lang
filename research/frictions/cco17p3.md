# Friction Report: cco17p3

The v0.2 port kept three parallel lists next to the x trie (`y_id: List<?Int>`, `y_tries`,
`y_totals`), padded them with `while ... push` loops, and wrote each bit walk as a `while` with a
`break`. Now strokes and queries are read as typed lists with `Coord = 1..=1e18`, each x node's
y trie and its colour sums are one `Column` struct, a `HashMap` `column_at` sends x nodes to columns,
`low_bits` is a comprehension over `bit_width(x)`, and one `walk` gives the trie path a
coordinate follows for both tries. The package gains tests.toml.

## A trie cannot carry a value per node
- **Where:** `Column.colour`, kept in step with `ys` by `colour.resize(ys.len, 0)`, and the
  `column_at` map beside `xs`.
- **Ideal:** `Trie<Bit, V>` with `t[node]` storage (or a node domain usable as `[t.Node]V`), so
  the x trie holds `?Column` and each column's trie holds its colour sums.

## Walking a key down a trie is written by hand
- **Where:** `fn walk`, which collects the nodes `step` reaches until it fails.
- **Ideal:** `t.path(word)` yielding the matched prefix's nodes.

## Dream
```cp
var xs = Trie<Bit, ?Trie<Bit, Int>>.new()
for s in strokes:
    let node = xs.add(low_bits(s.x))
    xs[node].get_or_insert(Trie.new())[xs[node]!.add(low_bits(s.y))] += s.v
for qu in queries:
    print(sum(sum(ys[p] for p in ys.path(low_bits(qu.c)))
              for o in xs.path(low_bits(qu.r)) if let ys = xs[o]))
```

- Tokens: port 436, original 939
- Max input (`gen.py --max`): port 1.33 s, 260356 KB, original 0.72 s, 157336 KB
