# ioi01p1

Score: 3/5
Dream: 3/5

Re-ported in v0.3 style (2D Fenwick tree).

## Cold review (HEAD)

Port 3/5 (provisional: the dream is under 4), dream 3/5. A std `Fenwick2D` with point add and rectangle fold (lines 14-23) is the whole editorial, and `X`/`Y` are separate domains.
- `let op: Int = read()` (line 17) and `else: break` (line 24) let any unknown code end the input. `let op: 1..=3 = read()` is possible today; the tagged-enum read is in the friction.
- The element range `-(1 << 30)..=1 << 30` (line 14) rests on the statement's 2^30 cap on the table total, which the header's Problem (lines 3-5) doesn't state (idiomatic-code/R1).

## Dream notes

- dream.cp writes match arms as `1:`/`2:`/`_:` (lines 8-14); `=>` is the one arm spelling (handbook/language/overview.md), so it caps at 3. It also folds both axes into one domain `X`, and the friction's description (commands as a tagged enum) doesn't match the file.
