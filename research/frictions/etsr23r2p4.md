# etsr23r2p4: School Canteen

A sliding-window bitmask DP. It reads close to the editorial; two small gaps.

## The two-layer relax is spelled twice
- **Where:** port lines 47 and 53: `x = min(x ?? cand, cand)` on a `?Int`
  cell, with the index written twice (`next[served.shifted(-1)][slid]`).
- **What felt wrong:** relaxing a DP cell is one idea; the repeated index is
  where a typo would hide.
- **Ideal code:** `next[served.shifted(-1)][last.prev!].lower(time)`.
- **General fix:** a std `?T` relax method (`lower` / `raise`); same gap as
  etsr23r2p2.
- **Severity:** low.

## Offsets from a moving origin
- **Where:** `fn at(i, j) = Pos(i.ord + j)` (line 23) turns a window offset
  back into a student.
- **What felt wrong:** the window is a sliding view `queue[i..]` keyed by
  `Offset`; today the port converts by hand.
- **Ideal code:** `let window = queue.window(i, Offset)` with
  `window[j]: ?Student`.
- **General fix:** a keyed window view over a domain-indexed array.
- **Severity:** low.

## Dream

```cp
@!multitest
let n: 1..=1000 = read()
struct Student:
    taste: 0..=1000
    patience: 0..=7
domain Pos = 0..n
let queue: [Pos]Student = read()
domain Offset = 0..8
domain Rel = -8..8
type Window = Subset<Offset>

fn choices(i: Pos, served: Window) -> yields Offset:
    var reach = Int(Offset.last!)
    for (j, s) in queue.window(i, Offset).some_entries.take_while(j => Int(j) <= reach):
        if served.contains(j): continue
        yield j
        reach min= Int(j) + s.patience

var cur: [Window][Rel]?Int = None
for j in choices(Pos.first!, Window.empty): cur[Window.single(j)][Rel(Int(j))] = 0
for i in Pos:
    var next: [Window][Rel]?Int = None
    for served in Window:
        for (last, time) in cur[served].some_entries:
            if served.contains(Offset.first!):
                next[served.shifted(-1)][last.prev!].lower(time)
            else:
                for j in choices(i, served):
                    cur[served.with(j)][Rel(Int(j))].lower(time + (queue[i + last].taste ^ queue[i + j].taste))
    cur = next
print(cur[Window.empty].min()!)
```
