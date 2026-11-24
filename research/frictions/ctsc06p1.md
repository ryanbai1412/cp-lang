# Friction report: ctsc06p1

The port is the dream minus two spellings: the expected wait is a sum over every border,
the whole name included, and reading the test count inline.

## `borders` leaves out the whole sequence

- **Where:** `let wait = Z(m) ** name.len + sum(Z(m) ** b for b in borders(name))`
- **What felt wrong:** the formula is one sum over all borders of the name; the port writes the
  whole-name term apart because `borders` yields proper borders only. Both forms are common
  (period problems want proper borders, expected-wait problems want all of them).
- **Goal it hurts:** express (a fact split in two)
- **Ideal code:** `name.borders(whole: true).sum(of: b => Z(m) ** b)`
- **General fix:** stdlib strings: a `whole:` option (default false) on `borders`, and the
  method form `s.borders()` beside the free fn.
- **Severity:** low

## Measurements

gen.py --max (t = 50 names of 1e5 letters): port 0.11 s, accepted C++ 0.21 s.

## Dream

```cp
// CTSC 2006 Singing Kingdom: letters 1..m drawn uniformly until the name appears. The expected
// wait is the sum of m^b over the borders b of the name, the whole name included; print it
// mod 10^4, four digits.
let m = read<1..=1e5>()
type Z = Mod<10_000>
for _ in 0..read<1..=50>():
    let name = read<List<1..=m>>()
    print("{name.borders(whole: true).sum(of: b => Z(m) ** b):04}")
```
