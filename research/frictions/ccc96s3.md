# Friction Report: ccc96s3

## A set of positions printed as a bit string
- **Where:** `print(Str([if ones.contains(i): '1' else: '0' for i in 0..n]))`.
- **What felt wrong:** the pattern is the indicator of `ones`; building it is a comprehension
  with a linear `contains` per position.
- **Goal it hurts:** brevity; costs are visible.
- **Ideal code:** `print(Bits(n, ones))` or `ones.indicator(0..n)`.
- **General fix:** an indicator/bit-string builder from a set of positions in `std.bitset`.
- **Severity:** low

## Dream

```cp
@!multitest
let n: 1..=30 = read()
let k: 0..=min(n, 7) = read()
print("The bit patterns are")
for ones in (0..n).combinations(k): print(Bits(n, ones))
print()
```

## Measurements

- Tokens: port 90, original 335
- Max input (`gen.py --max`): port 0.01 s, 3456 KB, original 0.05 s, 3456 KB
- Emitted C++: 9736 bytes
