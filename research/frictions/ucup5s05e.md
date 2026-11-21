# Friction report: ucup5s05e (Universal Cup 5 Stage 5 E)

Rewrote `proto/ports/ecnerwala/ucup5s05e.cp` to replace its hand-rolled monotone-chain `fn chain(side: Int)` and `type Site = (p: Pt, id: Int)` wrapper with `domain P = 1..=n` and `std.geometry.{convex_hull_idx, turn}` (`convex_hull_idx(pts, collinear: true)`).

- Gate: `proto/ports/ecnerwala/gate.sh ucup5s05e 50` passed (`PASS ucup5s05e (0 samples, 50 random cases)`).
- Tokens: `702 -> 625` (`-77` contestant tokens; `0.05x` raw original, `0.44x` stripped).
- Emitted C++: `25,054 -> 24,819 B` (`-235 B`).

## Splitting `convex_hull_idx` back into lower and upper x-monotone chains takes 5 lines of index math

- **Severity:** low
- **Goal it hurts:** express
- **Area:** geometry (`plan/STDLIB.md`)
- **What happened:** The port needs x-monotone lower and upper hull chains with original point IDs (`P`) so it can binary-search each chain by query `x`. `std.geometry` provides `convex_hull_idx(pts, collinear: true)` (which computes both chains internally and concatenates them) and value-only `lower_hull`/`upper_hull`, so the port finds the rightmost index `rm` and slices `hull[..=rm]` and `[hull[0]] + hull[rm..].rev` back out.
- **General fix:** Keep `convex_hull_idx` as the single indexed hull primitive for now (5 lines is cheap and avoids adding `_idx` twins for `lower_hull`/`upper_hull`); revisit only if multiple ports need indexed half-hulls.
