# Friction Report: canada21p5

## std `Mat` multiplies through dynamic function values
- **Where:** `fn mul` and `pow(step, k, op: Monoid(one, mul))` instead of
  `pow(Mat.new(step), k, op: Mat<Z>.product(Off.len))`.
- **What felt wrong:** `Mat.mul` keeps the semiring's `add`/`mul` as values, so `cpl explain`
  shows two `std::function`s in the hot loop; the port spells the product over its own domain.
- **Goal it hurts:** zero-cost abstractions; one spelling per idea.
- **Ideal code:** `print(step.pow(k)[Off.first!, Off(d)] / comb.choose(n, d))` with `step: [Off, Off]Z`.
- **General fix:** monomorphize `Mat`'s semiring (a type parameter, not stored functions), and
  give square `[D, D]T` arrays a matrix `pow`.
- **Severity:** medium

## Dream

```cp
let n: 1..=128 = read()
let m: 0..=n = read()
let k: 0..=1e9 = read()
let (s, t): (Str, Str) = read()
domain Off = 0..=n
let comb = Comb<Mod<1e9 + 7>>(n)
let step: [Off, Off]Mod<1e9 + 7> = [Off, Off] (i, j) =>
    sum(comb.choose(i.ord, c) * comb.choose(n - i.ord, m - c) for c in 0..=min(i.ord, m)
        if i.ord - c + (m - c) == j.ord) / comb.choose(n, m)
let d = (0..n).count(i => s[i] != t[i])
print(step.pow(k)[Off.first!, Off(d)] / comb.choose(n, d))
```

## Measurements

- Tokens: port 340, original 1108
- Max input (`gen.py --max`): port 0.12 s, 3400 KB, original 0.09 s, 3968 KB
- Emitted C++: 21539 bytes
