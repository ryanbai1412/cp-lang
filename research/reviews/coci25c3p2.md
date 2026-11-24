# coci25c3p2

Score: 3/5
Dream: none

The header identifies boxes with permutation cycles (lines 4–6), and line 18, `print(stirling1_row<P>(n)[k])`, directly expresses the resulting Stirling-number count (idiomatic-code/R2–R3). Input bounds are carried by `let n: 1..=5000 = read()` and `let k: 1..=n = read()` (lines 16–17; idiomatic-code/R5). The code is concise, but the missing dream caps the port at 3.

## Checklist failures
- Porting process: the required dream is missing; there is neither `dream.cp` nor a friction `## Dream` section. No code-level checklist failure is established.

## Owed rewrites
- Supply the required dream stating the cycle-counting argument; no port-code rewrite is established.

## Language gaps
None.

## Dream notes
None: there is no `dream.cp` and no `## Dream` in the friction report, so the port is capped at 3. The port's `print(stirling1_row<P>(n)[k])` (line 18) is what a dream would say.
