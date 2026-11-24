# cmt21peter2 frictions

## Dream

```cp
let at: [P](x: F64, y: F64) = read()
let total = [P] p => mean(K.turns(half: true).map(dir => sum(abs(shadow(dir, p) - shadow(dir, q)) for q in P))) * PI / 2
```

with the per-direction sum still computed by one sort and a running prefix.

## No statement bound for n

The statement page is behind a browser check, and neither the port nor the accepted C++
states a bound, so `n` stays `Int`; `gen.py --max` uses 2e5 points. The answer is not unique
(a few percent are allowed), so `checker.py` accepts within 3%.

## Running sum over a sorted order

`behind` is a hand-carried prefix of the sorted shadows; a `prefix` over the sorted list
would need the order's values copied into a second list first.
