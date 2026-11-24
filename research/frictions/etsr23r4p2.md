# etsr23r4p2: Encounters Are Greetings (power towers under range raises)

The dream builds the totient chain with `iterate(...).take_while(...)`, keys it
by its own domain `Step`, and leaves the generalized Euler theorem to an `Exp`
type; the port keeps a `List` of moduli, an `Int` depth and a hand-written
`power`.

## No exponent type for the generalized Euler theorem
- **Where:** port `fn power(e, big, m)`: the `(big, val)` pair and the 64-step
  loop that decides whether `base^exp` reached `m`.
- **What felt wrong:** "is the true value at least m" travels by hand next to
  every residue; one wrong `big` silently gives a wrong residue.
- **Goal it hurts:** hard-to-get-wrong, express.
- **Ideal code:** `Exp.pow(base, below, mod: m)` with `Exp(x, mod: m)`.
- **General fix:** an `Exp` (residue plus "reached the modulus") in std/math
  with `pow` applying generalized Euler.
- **Severity:** medium.

## A chain's keys are not a domain
- **Where:** `var moduli = [p]; while ...: moduli.push(phi(...))`, then
  `domain Slot = 0..=moduli.len` and `tower(s, x, depth: Int)` clamping
  `min(depth, moduli.len - 1)`.
- **Ideal code:** `let chain = iterate(p, phi).take_while(m => m > 1).chain([1]).list()`,
  `domain Step = chain.keys`, `depth.next ?? depth`.
- **General fix:** `iterate`/`take_while` producing a list, and `domain D = xs.keys`.
- **Severity:** low.

## A saturating shift action is spelled by hand
- **Where:** `Action(..., apply: (k, x, _) => [Slot] s => x[Slot.at(Int(s) + k) ?? Slot.last!])`.
- **Ideal code:** `act: shift_saturating`.
- **General fix:** a `shift_saturating` preset action over `[D]T` values.
- **Severity:** low.

## The judge's bounds are narrower than the statement's
- **Where:** gen.py `modulus()`: the accepted C++ caches phi below 1e7 and keeps
  11 tower levels, and it loops forever on c = 1, so generated inputs keep
  p < 1e7 with a totient chain of at most 10 and c >= 2, while the port's
  types keep the statement's 1..=1e8.
- **General fix:** record judge-observed bounds in the package (`meta.json`)
  so gen.py and the port's types can both cite them.
- **Severity:** low.
