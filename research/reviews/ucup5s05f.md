# ucup5s05f

Score: 3/5
Dream: 3/5

Scored cold against `design/EXEMPLARS-PORTS.md` (previous score 2/5; awaiting rescore).

## Language gaps

Logged in `research/frictions/ucup5s05f.md`:
- `std.poly`: `SubproductTree` / transposed multipoint evaluation (`eval_transposed`).
- `std.strings`: suffix-tree / LCP-interval edge walk (`suffix_tree_edges`).

## Cold review (HEAD)

Dream 4/5 (new), port 2 → 3/5, rescored after e0f2c521. The header derives the power-sum and transposed-evaluation route, and the program is that route.
- `SubproductTree` and `suffix_tree_edges` (lines 7, 9) are not in the stdlib; the friction reports cover them.
- Minor: `above` holds `bottom_size` and `below` holds `top_size` (lines 24-25), so the names are inverted. `inv_fact(i + 1)` (line 32) doesn't match the header's `/ k!` without explanation.

Port 3/5: it hand-writes the subproduct tree (`product`/`descend` with `2 * node` indexing) and the suffix-tree edge walk, both reported gaps.

## Cold review (HEAD)

Port 3/5, rescored cold on current main; dream 4/5 holds. The port checks with zero errors. The header derives the power-sum route. `product`/`descend` (lines 17-42) hand-write the reported subproduct tree on raw heap indices, and the port stays at 3 until it lands.
- Minor: `suffix_tree_edges` (lines 45-72) handles each finished edge twice (lines 56-59, 63-66); name that once.

## Cold rescore (bc366c27f)

Port 3/5 (reported, measured gap), rescored cold after the re-port ae9d57012; dream 4 → 3/5. `cpl check` and `cpl lint` are clean (0.3 and `CPL_EDITION=0.2`), `cpl test` passes.

Dream re-judged: its closing formula (line 6) is not derived, and the call that is meant to compute it, `tree.eval_transposed(sums)` (lines 29-30), goes the other way (weights to power sums, per its std doc and the friction file), so the dream's key line is wrong; it also hand-writes `power_sums` (lines 13-17), drops the alphabet (line 21), swaps `above`/`below` (lines 23-24) and shifts `inv_fact(i + 1)` unexplained (line 31). The dream should be the port's lines 28-33 with the editorial line `let answer = compose(egf, log([Z(1), Z(1)], n + 1), n + 1)`, `egf[j] = p[j] / j!`, derived in the header from `sum_s (1 + y)^s = sum_j p[j] log(1 + y)^j / j!`. The port is judged against that.

Better:
- The subproduct tree on `2 * node` indices and the monotone-stack walk are gone: `suffix_tree_edges` and `power_sums` from the stdlib (lines 30-32), and the names now match (`bottom`, `top`).
- The header (lines 3-7) derives the route: the per-edge telescope, why power sums, and the weights on `0..=n` read off a Taylor shift; lines 14-16 derive the weights by partial fractions.
- `s: Str of "abcdefghijklmnopqrstuvwxyz"` (line 29) keeps the alphabet; the missing length bound is the logged E0504.

Still fails:
- Hand-rolled standard algorithm (lines 17-25): `binomial_sums` is transposed interpolation on `0..=n` written out (the `[deg + 1..]` slice of a convolution, line 21; `1 / q'(m)` spelled as sign times two inverse factorials, lines 23-24). The editorial `compose` line runs 2.1x orig.cpp (`compose` 1.87 s at n = 2e5) and `SubproductTree.interpolate_transposed` is missing; both reported and measured, which is the definition of a 3. It reaches 4 when either lands.
- Owed, header: lines 1-7 are not the `// Source:` / `// Problem:` / `// Solution:` blocks (`corpus/header.py` lists the port).
- Owed, perf: ae9d57012 measured 1.38x orig.cpp on max random `ab` input (2.04 s vs 1.48 s), over the 1.1x bar, and no perf friction names it (the filed one is `compose`'s 2.1x).
- Minor (probed, same output as the port on `gen.py` inputs): the `sign` closure (line 23) is `(-Z(1)) ** (deg - m)` inline, and `print(answer[1..=n])` prints `Mod` without `.map(a => a.v)` (line 34).

## Cold rescore (fd5c0ca35)

Port 3/5 (reported, measured gap), rescored cold after the re-port 481cce81f; dream 3/5 holds (unchanged, so the port is still judged against the `compose` rewrite named above). `cpl check` (0.3 and `CPL_EDITION=0.2`) and `cpl lint` are clean, `cpl test` passes 1/1.

Better:
- The header (lines 1-10) is now the `// Source:` / `// Problem:` / `// Solution:` blocks with the cost, closing the owed header.
- The perf gap has its own friction ("Slow: the port runs 1.44x"), profiled to `power_sums` (lines 35-36) and the two trees over `0..=n` (lines 23, 25). A spot run of the built port on max random `ab` input took 1.43-1.54 s, matching its 1.43 s.
- The sign is `(-Z(1)) ** (deg - m)` inline (line 27), and `print(answer[1..=n])` prints the `Mod` list directly (line 38).

Still fails:
- Hand-rolled standard algorithm (lines 17-29): `binomial_sums` is transposed interpolation on `0..=n` written out by partial fractions. The editorial `compose` line is measured at 2.1x and `SubproductTree.interpolate_transposed` is missing; both are reported and measured, which is a 3. It reaches 4 when either lands and the helper becomes one line.
- Minor, closing formula: line 27's `(-Z(1)) ** (deg - m) * inv_fact(m) * inv_fact(deg - m)` is `1 / q'(m)` on these points, but the comment (lines 17-19) stops at `num(m) / q'(m)`; add "here q'(m) = (-1)^(deg - m) m! (deg - m)!".
- Minor, reported: lines 23 and 25 build two subproduct trees over the same points; `SubproductTree` does not expose its root, so one shared tree needs the std change the perf friction names.
- Minor: `cpl build --judge codeforces` gives 40979 bytes at this main, not the 40320 the size friction cites; refresh the number.
