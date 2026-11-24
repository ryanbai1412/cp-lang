# Friction Report: noip19p1

No gaps: `U64` holds k < 2^64, so the v0.2 port's two 32-bit halves are gone; the answer is `k ^ (k >> 1)` printed bit by bit.

## Dream

```cp
let n: 1..=64 = read()
let k: U64 = read()
print("{k ^ (k >> 1):0{n}b}")
```

## An n-bit binary string of a `U64` is printed bit by bit
- **Where:** `print([if (gray >> i) & U64(1) == U64(1): '1' else: '0' for i in (0..n).rev])`.
- **What felt wrong:** the answer is `k ^ (k >> 1)` written in n binary digits; the port builds the string one shifted, masked bit at a time.
- **Goal it hurts:** express.
- **Ideal code:** the dream's `"{k ^ (k >> 1):0{n}b}"`.
- **General fix:** the `:b` format for `U64` (today E0302: "`:b` is not a number format this value takes: it is a `U64`") and a width taken from a value (`:0{n}b` is E0107). Owner: Core (12cfdb46).
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5127723; max (n = 64, k = 2^64 - 1) same output.
