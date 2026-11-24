# ceoi18p5 frictions

## Dream

```cp
let n: 1..=1e9 = read()
let found = factorisations(n, min_factor: 2).map(f => f.sum - f.len).to_set()
print(found.len)
print(found)
```

## Factorisations by hand

Std has `divisors` but no enumeration of multiplicative partitions, so the port keeps the
recursive split over non-decreasing divisors with an index into the divisor list.
