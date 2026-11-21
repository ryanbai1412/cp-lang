# 24. Modular arithmetic

**Layer:** library. `Mint(mod: p)` is a scoped type (chapter 12) whose arithmetic impls are library code over the `mulhi` intrinsic; a constant modulus specializes. Status: [proposed].

## 24.1 Modular arithmetic

- `type Z = Mint(mod: 998244353)`: specialized, constant modulus. `type Z = Mint(mod: p)` with `p: Prime`
  read from input: a fresh scoped type with Barrett reduction (chapter 12.3). One local edit between them.
- `?Z` is free: values ≥ the modulus are the niche.
- `Z(x)` converts from `Int`, reducing negatives by floor. `z.inv()` and `/` exist only for a prime modulus;
  the `Prime` type checks primality at the read in **both** builds (one deterministic Miller–Rabin) [r11 N24].
- Storage: modulus `< 2^31` → `u32`; larger → `u64` with an `I128` multiply, which the profile must allow
  [r11 N25].

- **Combinatorics** [p2 F8]: `Z.factorials(n)` builds `fact`/`inv_fact` with one `inv` and a backward pass; `f.binom(n, k)` is
  0 outside `0 ≤ k ≤ n`; `cp explain` shows the two `[0..=n]Z` tables.
- **Carries** [p2 F10]: `overflowing_add`/`overflowing_sub -> (T, Bool)`, `U64.lo32()`/`.hi32()`.

## 24.2 Round-3 rules [p3]

- Arithmetic between equal fixed widths stays in that width (11.3 wrap/trap rules), so hashing and bit tricks
  work. Mixing with `Int` widens losslessly (`I32`/`U32` to `Int`; `U64` with `Int` is an error) and gives
  `Int`; storing back needs `I32(x)`.
- `Int(z)` on a `Mint` is the residue in `0..mod` (total).
- Stdlib: `pow_mod(b, e, m)`, `n.prime_factors()`, `gcd`/`lcm` on every integer width including `I128`,
  printing `I128`, `x.sign()`, `abs(x)`, and `Ratio<T>` (reduced, with `num`/`den`).
