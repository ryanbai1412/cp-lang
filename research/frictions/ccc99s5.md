# Friction Report: ccc99s5

## Backtracking with undo is spelled by hand
- **Where:** `fn fill(p: At, give: ?(x: Letter, d: Int) = None)`, setting and resetting
  `digit[x]` and `used[d]`.
- **What felt wrong:** "try this choice, recurse, undo on failure" is written out with paired
  assignments; the natural continuation form (`give(x, d, rest: () -> Bool)`) lowers to a
  `std::function`, so the pending choice rides along as an optional parameter instead.
- **Goal it hurts:** brevity; costs are visible.
- **Ideal code:** `with digit[x] = Some(d), used[Digit(d)] = true: fill(next)` (scoped
  assignment undone on exit), or `give(x, d, () => fill(next))` with the closure inlined.
- **General fix:** inline closure parameters of non-escaping nested fns (Core), or a scoped
  `with` assignment for backtracking state.
- **Severity:** medium

## Dream

```cp
domain Letter = 'A'..='Z'
domain Digit = 0..10
domain Row = 0..3
@!multitest
let words: [Row]Str = read()
var digit: [Letter]?Digit = None
let leads = Set(Letter(w[0]) for w in words)
fn letter(r: Row, col: Int) -> ?Letter = words[r].rev.get(col).map(Letter)
fn give(x: Letter, d: Digit, rest: () -> Bool) -> Bool:
    if let old = digit[x]: return old == d and rest()
    not (digit.contains(Some(d)) or (d.ord == 0 and x in leads)) and
        with digit[x] = Some(d): rest()
fn fill(col: Int, row: Row, total: Int) -> Bool: ...
assert fill(0, Row.first!, 0)
for w in words: print(Str(w.map(c => Char('0' + digit[Letter(c)]!.ord))))
print()
```

## Measurements

- Tokens: port 554, original 722
- Max input (`gen.py --max`): port 0.00 s, 3072 KB, original 1.11 s, 3456 KB
- Emitted C++: 17542 bytes
