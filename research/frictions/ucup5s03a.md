# ucup5s03a friction

## No series type: substitution, shift and division are index arithmetic

- **Goal it hurts:** express
- **Severity:** medium
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s03a.cp:21,25-27`
  (`Fps.egf(...).binomial_transform()`, `X * t00.of(X ** 2)`, `(s / a)[n]`)
- **Actual:** `proto/ports/ecnerwala/ucup5s03a.cp:20-40`
  convolves with `inv_fact` by hand, reads `x^shift f(x^2)` through `spread`,
  and divides with `convolve(s, inv(a, n + 1))`.
- **General fix:** an `Fps<Z>` value type in `std.poly` with ring operators,
  `X`, substitution `f.of(X ** k)`, EGF construction and `/` truncated to the
  needed length, built on the existing `convolve`/`inv`.

## Measurement

`(n, k)` stay `(Int, Int)`: the statement's bounds are not recorded. n =
200,000, k = 150,000, `g++ -O2`: 0.21-0.25 s vs 0.19-0.35 s, same output.
Emitted C++ 26,839 -> 27,580 bytes.
