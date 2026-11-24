# Friction Report: iati21a12

## Dream

```cp
var supply: [V]MeldHeap<(value: Int, count: Int)> = [V] _ => MeldHeap(by: l => -l.value)
supply[v].add_all(value: w)        // lift one field of every element
supply[u].absorb(mut supply[v])
```

## Gaps

- **`add_all` needs `T: Add` on the whole element.** Lifting just the value
  of a `(value, count)` lot takes a `struct Lot` whose `+` adds values and
  keeps the left count, which is not a real addition. A keyed lift
  (`add_all` on the `by:` key) would remove it. Severity: medium.
- **Amount and weight bounds are untyped.** The statement isn't public; the
  generator keeps |w| <= 1e4 and amounts <= 1e4, but the port can't claim
  them. Severity: low.
