# ioi14p1: Rail

## Dream

```cp
extern fn getDistance(i: Station, j: Station) -> Int
@grader fn findLocation(n: 1..=5000, first: Block, location: mut [n]Block, stype: mut [n]1..=2):
    ...
```

## A grader fn can't fill the judge's arrays
- **Where:** the port reads the hidden stations from stdin and simulates
  `getDistance`; `@grader fn findLocation(..., location, stype)` is E0501, so the
  port can't be submitted as written. `corpus/problems/ioi14p1/orig.cpp` wraps the
  accepted C++ in the same sample grader for comparison.
- **General fix:** `mut` array params on grader fns lowered to the judge's
  pointers.
- **Severity:** high.

## Block positions as distances go through `.ord`
- **Where:** `get_distance`: `right_d[a]!.ord - left_c[b]!.ord`; there is no
  difference of two domain values.
- **General fix:** `x - y` on two values of one ordered domain giving `Int`.
- **Severity:** low.
