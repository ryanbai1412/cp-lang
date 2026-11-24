# btoi19p2 frictions

## Dream

```cp
enum Signal: N = 'N' | E = 'E' | S = 'S' | W = 'W' | Any = '?'
let signals: [m]Signal = read()
let sea = Bitset<(Row, Col)>.from([(Row, Col)] p => map[p] == '.')
let end = signals.fold((at, s) => (s.moves.map(d => at.shifted(d)).union()) & sea, sea)
print(end.len)
```

## Signals as characters

Reading the signals as an enum needs each variant's input spelling, so the port reads `Char`
and asks `heard(sig, 'N')`. A grid bitset over `(Row, Col)` with a shift by a grid step would
replace the per-row `[Row]Bitset<Col>` table and its row-above/row-below cases.
