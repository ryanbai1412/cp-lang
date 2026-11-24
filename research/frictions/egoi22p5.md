# Friction report: egoi22p5

The hand-written two-pointer merge and the copy back are now std `merge_sorted`. The counts stay ascending, so the top centres are the tail and no sort key is needed; the answer prints `free.rev`.

Gap (speed): `merge_sorted` builds a new list every service, about 3x the accepted C++ at max. With `by: x => desc(x)` it was 4.8 s, so I sent Core a repro for the `Desc` key not inlining. Ideal: an in-place merge of two adjacent sorted runs.

## Dream

```cp
for x in free[cut..n]: x -= take
free.merge_runs(cut)  // in place, the two sorted runs free[0..cut] and free[cut..n]
```

## Measurements

gen.py --max (n = 1e5, s = 5000): port 2.24 s, v0.2 port 1.55 s, accepted C++ 0.77 s.
