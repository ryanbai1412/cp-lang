# Friction report: nccc10s4

## Dream

```cp
let n: 3..=12 = read()
domain L = 0..n
let lines: [L]Line2 = read<[L](a: Point, b: Point)>().map(e => Line2.through(e.a, e.b))
let corners = Arrangement(lines)                // exact crossings, one id per point, in order per line
var used: [L]Bool = false
var spent: [corners.Point]Bool = false
fn walk(at: corners.Point, start: corners.Point, sides: Int) -> Int:
    if sides == n: return 0
    sum(for i in L if not used[i] and let here = corners.place(at, i):
        with used[i] = true:
            sum(for run in corners.along(i).both_ways(from: here):
                run.take_while_unspent(spent).sum(c => Int(sides + 1 == n and c == start) + walk(c, start, sides + 1))))
print(sum(for (k, s) in corners.along(L.first!).items: ...))
```

The dream.cp that used to sit here solved a different reading (3 or 4 segments) and did not
check; it is replaced by this ideal.

## No exact arrangement of lines

`Line`, `Hom`, `crossing`, the `Ord` on homogeneous points and the corner numbering
(`Map<Hom, Int>` then `domain Corner`) are all hand-written. A std exact line arrangement
(rational crossings, a point id domain, the ordered corners on each line, and where a corner
sits on a line) would remove most of the port.

## Walking both ways along a line

`for dir in [1, -1]` with `along[i].get(k)` and a second loop to un-spend the passed corners
is the only spelling of "walk outwards from here in both directions until blocked". A view
`along[i].both_ways(from: here)` (two runs) plus a scoped `with spent[c] = true:` that undoes
itself would make the backtracking state impossible to leak.

## Domains built from a map's size

`domain Corner = 0..id.len` works only after the map is complete; corner ids are `Int` until
then and converted with `Corner(id[p])`.

## Verification

30 random cases (n <= 7) agree with the v0.2 port, and 30 (n <= 7) plus 10 (n <= 9) with the accepted C++
(`corpus/submissions/4225209.cpp`). The max case (12 lines, |coordinate| <= 2000)
gives the same answer as the C++ in 4.9 s, the v0.2 port's time; the C++ takes 2.6 s. The
emitted C++ is 37 KB, over the 32 KB generic judge target (W0201).
