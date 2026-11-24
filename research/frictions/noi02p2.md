# Friction report: noi02p2

## Dream

```cp
let right = [Child] p => ask.says_star(p, p.ring_next)
let referee = Child.find(p => right[p] != left[p.ring_next])!
```

## Changes

The v0.2 port kept children and roles as bare `Int`s (`(p + 1) % n`, roles 0/1/2, `wild = 0`
then a break loop, a `while p >= 0` walk). The rewrite makes children a domain with
`Child.wrap` for ring neighbours, answers `Bool`s, roles an enum, finds the referee with
`Child.find`, and walks `(..referee).rev` and `Child.after(referee)`.

## Remaining friction

- No ring successor on a domain: `Child.wrap(Int(p) + 1)` spells it.
- `ask` returns raw numbers, so the answer is read as `0..=1` and compared to 1.
- The bounds come from `ask`, which the `// Bounds:` gate does not read, so the header has none.

## Verification

30 generated cases pass the interactor (the problem has many valid question orders, so the
interactor judges instead of comparing with the v0.2 port). Corpus tests (8) pass.
