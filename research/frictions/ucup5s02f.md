# Friction report: ucup5s02f

`dream.cp` is verified. The port is the dream with two differences:
- the reads carry the statement's bounds (n, m <= 50, q <= 100, ends <= 1e18) where the dream reads 1..=200_000 counts and bare `(Int, Int)` pairs;
- every integer literal mixed with an `I128` is written `I128(6)`, `I128(1)` and so on.

## Integer literals don't take the `I128` type of their context
- **Where:** port lines 36-46: `I128(6) * k`, `hi / t6 + I128(1)`, `covers(k, I128(6), p0, t)` and `k *= I128(6)`. The dream writes `6 * k`, `+ 1`, `covers(k, 6, p0, t)` and `k *= 6`, and compiles.
- **What felt wrong:** a literal next to an `I128` already has only one sensible type. Wrapping each one hides the arithmetic.
- **Goal it hurts:** express.
- **Ideal code:** the dream's bare literals, which today's checker accepts. The port's wrappers were written for an older checker. The gap is now in the port, and the dream's bounds still need porting into the dream.
- **General fix:** none needed in the language. Literal adaptation works; the port's `I128(...)` wrappers are owed spelling. A lint for "a literal conversion the context already implies" would catch this class (`I128(6)`, `F64(1)`). Owner: lint.
- **Severity:** low.

## The header repeats the Solution
- **Where:** the 18-line comment block after `// Solution:` restates the same derivation and the 6/3/2 refinement rule.
- **What felt wrong:** one spelling per idea. The second block is the v0.2 header and the first is its summary.
- **Goal it hurts:** revision.
- **Ideal code:** the `// Solution:` paragraph alone, with the inequality `6kl - tv <= 6ti <= 6kr - tv` folded into it.
- **General fix:** none; header hygiene. Owner: idioms.
- **Severity:** low.

## Measurements

- **Tokens:** port 423, accepted C++ 451 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.01 s / 3 MB, accepted C++ 0.00 s / 3 MB, with the same output.
- **Emitted C++:** 15549 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
