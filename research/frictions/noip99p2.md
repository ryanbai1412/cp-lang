# Friction report: noip99p2

There is no dream.cp; the ideal is below. The port simulates on a digit list. Adding a number to its reversal is `zip(d, d.rev)` with a carry, and `steps() -> ?0..=30` makes "more than 30" the `None` case.

## Dream

```cp
let base: 2..=16 = read()
let d = digits(read<Str>(), base:)        // most significant first
fn steps() -> ?0..=30: ...
```

## Base-N digits are parsed by hand
- **Where:** line 25: `[if c.is_digit: c - '0' else: c - 'A' + 10 for c in number]`.
- **What felt wrong:** reading a base-16 numeral spells the ASCII arithmetic out. Lowercase hex or a digit at or above `base` would be accepted silently.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:** `digits(number, base:)` returning `List<0..base>`, rejecting out-of-range characters.
- **General fix:** a std parse of a numeral string into base-N digits, the inverse of the `digits(x, base:)` used for output. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 166, accepted C++ 419 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 3 MB, accepted C++ 0.00 s / 3 MB, with the same output.
- **Emitted C++:** 15793 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
