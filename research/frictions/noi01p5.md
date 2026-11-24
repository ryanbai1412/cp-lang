# Friction report: noi01p5

## Dream

```cp
let (left, right) = T.split(n / 2)
print(count_pairs(sums(left), sums(right), (x, y) => x + y == 0))
```

## Sorted halves instead of a counts map

The v0.2 port put the 3.4e6 first-half sums into a `counts` map and peaked at 344 MB on the
max case. The rewrite sorts the first half and walks the sorted second half with
`equal_range`. That peaks at 88 MB and runs in 0.72 s; the accepted C++ takes 36 MB and 0.29 s.
The halves come from `partition` instead of two filtered comprehensions, and the unused
`pow` helper is gone.

## Verification

`gen.py` is new (the max case is n = 6, M = 150, every p = 3). 30 random cases agree with the
v0.2 port and 30 with the accepted C++. Corpus tests pass 6/6, including the max case,
which was MLE before.
