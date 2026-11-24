# Friction Report: noip00p1

## 1. No digit-to-character for a base
- **Where:** `digits.push(if d < 10: '0' + d else: 'A' + (d - 10))`.
- **What felt wrong:** the alphabet 0-9A-Z is spelled by hand for every base conversion.
- **Goal it hurts:** express.
- **Ideal code:** `Char.digit(d)`; `x.digits(base: base)` accepting negative bases.
- **General fix:** `Char.digit(d: 0..36)` and `Int.digits(base:)` for any base with |base| >= 2.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5067849 on gen.py inputs; max (x = 32767, base -2) 0.004 s vs C++ 0.003 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
let x = read<-32768..=32767>()
let base = read<-20..=-2>()
print(x.digits(base: base).rev.map(Char.digit))
```
