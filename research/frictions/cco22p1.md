# cco22p1 friction

## Alternating pairs need a parity of positions

- **Where:** `port.cp`, `consistent`: the pairs of `line[x..=y]` alternate in direction.
- **What felt wrong:** `line[x..=y].adjacent` gives the pairs but there is no `enumerate` on the
  list it yields (E0203 "a `List<(Student, Student)>` has no member `enumerate`"), so the port
  walks the positions and takes the parity as `(i.ord - x.ord) % 2`.
- **Goal it hurts:** express.
- **Ideal code:** `for (i, (a, b)) in line[x..=y].adjacent.enumerate: if i % 2 == 0: ...`
- **Severity:** low.

## Dream

```cp
fn consistent(x: Spot, y: Spot) -> Bool:
    var taller: [Student]List<Student> = []
    for (i, (a, b)) in line[x..=y].adjacent.enumerate:
        if i % 2 == 0: taller[a].push(b)
        else: taller[b].push(a)
    topo(taller) != None
```

## Measurements

30 generated cases agree with the old port and 30 with the accepted C++. The binary-search
form (`(x..=Spot.last!).last_true(y => consistent(x, y))`) ran 1.38 s against the accepted
C++'s 0.24 s on N = K = 3000 distinct students; the two-pointer form makes O(N) checks instead
of O(N log N) and runs 0.14 s. The `--max` input matches the accepted C++.
