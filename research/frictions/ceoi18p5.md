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

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
let n: 1..=1e9 = read()
let found = factorisations(n, min_factor: 2).map(f => f.sum() - f.len).to_set()
print(found.len)
print(found)
```

```text
error[E0201]: no `factorisations` in scope, and not in the stdlib (yet)
  --> probe.cp:2:13
2 | let found = factorisations(n, min_factor: 2).map(f => f.sum() - f.len).to_set()
  |             ^^^^^^^^^^^^^^
  = help: declare it first: `let factorisations = todo`
```

**Status:** open.
