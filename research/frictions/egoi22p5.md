# Friction report: egoi22p5

The hand-written two-pointer merge and copy back are now std `merge_sorted`, on just the window where the two sorted runs overlap (found with two `first_true` binary searches). The counts stay ascending, so the top centres are the tail and no sort key is needed; the answer prints `free.rev`.

A full `merge_sorted` per service took 2.2 s at max (TLE in `cpl test`), and 4.8 s with `by: x => desc(x)`. I sent Core a repro for the `Desc` key not inlining.

## Dream

```cp
for x in free[cut..n]: x -= take
free.merge_runs(cut)  // in place, the two sorted runs free[0..cut] and free[cut..n]
```

## Measurements

gen.py --max (n = 1e5, s = 5000): port 0.20 s, v0.2 port 1.55 s, accepted C++ 0.77 s.
