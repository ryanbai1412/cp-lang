# ioi14p2: Wall

## Dream

```cp
@grader fn buildWall(n: 1..=2e6, k: 1..=5e5, op: [k]1..=2, left: [k]Col, right: [k]Col,
                     height: [k]Height, final_height: mut [n]Height):
    var wall = SegTree(max, [Col] _ => 0, act: Clamp.action)
    for i in 0..k: wall.apply(left[i]..=right[i], Clamp.of(op[i], height[i]))
    final_height = [Col] c => wall.get(c)
```

## The judge's function task has no CP Lang form
- **Where:** the port reads stdin; `corpus/problems/ioi14p2/orig.cpp` wraps the
  accepted C++ in a sample grader. A `@grader` fn can't fill `finalHeight`
  (the same E0501 as ioi14p1).
- **Severity:** high.

## Clamp composition is hand-written
- **Where:** `clamps`: the identity, compose and apply of an interval clamp.
  Clamping is a common lazy action (also `chmin`/`chmax` ranges).
- **General fix:** a std `Clamp` action next to `BeatsSegTree`.
- **Severity:** low.
