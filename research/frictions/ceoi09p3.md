# ceoi09p3 frictions

## Dream

```cp
let pts = read<[P]Pt>().sorted()
let (domain H, height) = compress([0] + pts.map(p => p.y))
```

## Sorting a table keeps a list

`read<[P]Pt>().sorted()` is a `List`, not a `[P]Pt`, so the port reads into a `var` table and
sorts it in place. `[0] + [p.y for p in pts]` also fails (`List<Int> + List<_>`); the port
writes `pts.map(p => p.y).collect()`.

## Reference fix

The accepted C++ compared the height index `m - 1` with heights for a zero-width rectangle and
printed its infinity on a tall isolated point; `orig.cpp` carries only that fix.
