# ioi10p6: Traffic Congestion

## Dream

```cp
@grader fn LocateCentre(n: 1..=1e6, fans: [n]1..=2e9, s: [n - 1]City, d: [n - 1]City) -> City:
    let tree = Rooted(Graph.undirected(zip(s, d)), City.first!)
    let below = tree.subtree_sums(fans)
    City.argmin(by: c => max(below.total - below[c], tree.children(c).max(k => below[k]) ?? 0))!
```

## The judge's function task has no CP Lang form
- **Where:** the port reads stdin; `corpus/problems/ioi10p6/orig.cpp` wraps the
  accepted C++ in a sample grader for comparison. A `@grader` fn can't take the
  judge's parallel arrays (the same E0501 as ioi14p1).
- **General fix:** grader fns over C arrays, with lengths from other params.
- **Severity:** high (the port can't be submitted to the judge as written).

## Subtree sums are a hand loop
- **Where:** `below`: a reverse preorder walk adding each child into its parent.
- **General fix:** `Rooted.subtree_sums(values)` in std.
- **Severity:** low.
