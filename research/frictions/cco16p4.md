# Friction Report: cco16p4

## Counting equal pairs takes a running Counter
- **Where:** `pairs += seen[key]; seen[key] += 1`.
- **What felt wrong:** "pairs of grids with equal keys" is `sum(k * (k - 1) / 2)` over a Counter
  of keys, but a Counter cannot be built from a lazy sequence of reads.
- **Ideal:** `Counter(key(read()) for _ in 0..g).pairs` (a `pairs` count on Counter).

## The key is two List<Int>
- **Where:** `([... % 2 for r in Row], [... % 2 for c in Col])`.
- **What felt wrong:** parities of rows and columns are `[Row]Bool` and `[Col]Bool`; Lists of Int
  are used only because domain arrays cannot be Counter keys today.
- **Ideal:** domain arrays as hashable keys.

## Dream
The committed `dream.cp` (canonical form by clearing the top-left block); the port keys by the
equivalent row and column parities.

- Tokens: port 178, original 639
- Max input (`gen.py --max`): port 0.01 s, 6952 KB, original 0.00 s, 7552 KB
