# Friction report: mccc3j5

## Dream

```cp
let restricted: [m](u: V, v: V) = read()
print(Subset<V>.filter(keep => restricted.none(e => {e.u, e.v} <= keep)).map(.len).max()!)
```

## Subset containment of a pair

"keeps both ends" is `e.u in keep and e.v in keep`. A pair-as-subset test (`{u, v} <= keep`)
would say it directly.

## `max` of a non-empty filter is still optional

The empty set always qualifies, yet `max(...)` over the filtered subsets is `?Int` and needs
`!`.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). The max case
(N = 18, all 153 pairs) runs instantly. Corpus tests pass.
