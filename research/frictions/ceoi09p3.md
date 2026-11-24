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

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
struct Pt:
    x, y: Int
let n: 1..=5 = read()
domain P = 0..n
let pts: [P]Pt = read<[P]Pt>().sorted()
print(pts[P.first!].x)
```

```text
error[E0302]: expected `[P]Pt`, found `List<Pt>`
  --> probe.cp:5:18
5 | let pts: [P]Pt = read<[P]Pt>().sorted()
  |                  ^^^^^^^^^^^^^^^^^^^^^^
```

**Status:** open.

```cp
struct Pt:
    x, y: Int
let n: 1..=5 = read()
domain P = 0..n
let pts: [P]Pt = read()
print([0] + [p.y for p in pts])
```

```text
0 1 2
```

**Status:** fixed: `[0] + [p.y for p in pts]` builds; the port's `.map(...).collect()` workaround is owed.
