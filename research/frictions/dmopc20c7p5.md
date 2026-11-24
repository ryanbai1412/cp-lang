# Friction report: dmopc20c7p5

## Two kinds of choice share a flat index space

- **Where:** `domain Choice = 0..n + m` with `fn city`, `fn road` offsets
- **What felt wrong:** a closure over cities and roads wants one domain whose elements are either; without sum domains the port offsets road ordinals by n by hand.
- **Goal it hurts:** ergonomic, correct (an off-by-n offset type-checks)
- **Ideal code:** `domain Choice = City + Road` and `match x: City(c) => -fee[c], Road(r) => roads[r].toll`
- **Repro (fresh build):** `domain A = 1..=3`, `domain B = 0..2`, `domain N = A + B`, then `match v: A(a) => ...` gives `error[E0302]: \`A\` is a variant, but the value matched is a \`N\``.
- **General fix:** core: sum domains (already planned in spec/lib/flows.md).
- **Severity:** medium

## Measurements

gen.py 1 500 (N = M = 500): port under 0.01 s, accepted C++ 0.41 s.

## Dream

The committed `dream.cp` (sum domain `Item + Pair`) is unchanged.
