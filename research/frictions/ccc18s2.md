# Friction Report: ccc18s2

## Strictly increasing has no std spelling
- **Where:** `fn increasing(line) = line.adjacent.all((a, b) => a < b)`.
- **What felt wrong:** `is_sorted` allows equal neighbours; the strict form needs the
  pairwise spelling.
- **Goal it hurts:** brevity.
- **Ideal code:** `line.is_sorted(strict: true)`.
- **General fix:** a `strict:` flag on `is_sorted`, as `lis` has.
- **Severity:** low

## Repeating a function until a test passes
- **Where:** `var g = given` / `while not grows(g): g = turned(g)`.
- **What felt wrong:** `iterate(from:, step:)` takes a `T -> ?T` step, so a total step needs a
  `Some(...)` wrapper; the loop is shorter.
- **Goal it hurts:** brevity.
- **Ideal code:** `let g = iterate(given, turned).find(grows)!`.
- **General fix:** accept a total `T -> T` step in `iterate`.
- **Severity:** low

## Dream

```cp
let n: 2..=100 = read()
domain I = 0..n
let given: [I, I]1..=1e9 = read()
fn grows(g: [I, I]Int) -> Bool = g.rows.each().all(r => r.is_sorted(strict: true)) and
    g.cols.each().all(c => c.is_sorted(strict: true))
for row in iterate(given, g => g.rotated_cw()).find(grows)!.rows.each(): print(row)
```

## Measurements

- Tokens: port 192, original 626
- Max input (`gen.py --max`): port 0.00 s, 3424 KB, original 0.00 s, 3584 KB
- Emitted C++: 15069 bytes
