# cco04p4

Score: 3/5
Dream: 3/5

Each orbit is read as its list of positions over one lap and the answer is one `min(... for t in 0..lcm(a.len, b.len))!`, with the lcm bound derived in the header; no sentinel, no running best.

## Checklist failures

- idiomatic-code/R4: `dist2` (line 34) writes each difference twice, `(p.x - q.x) * (p.x - q.x)`; `**` is the power spelling (decision §70) and runs under 0.2.

## Owed rewrites

- Lines 22-24 read the moves one `d`/`axis` at a time; the dream's `let moves: [n](d: -100..=100, axis: Axis) = read()` names them and puts the bound on the read.
- `fn dist2(p, q) = (p.x - q.x) ** 2 + (p.y - q.y) ** 2`.

## Language gaps

- The lap is built by pushing `at` once per unit step (`fn read_orbit`): research/frictions/cco04p4.md, "A lap of unit steps is built by hand".
- `min(...)!` over a range that is never empty: same report.

## Dream notes

- The dream header names another problem ("Gator Chase-ish") and has no Source/Solution lines.
- It keeps the C++ construction the friction itself calls the gap: a push loop per unit step instead of `moves.flat_map(unit_steps).scan(start, +)` (caps at 3). The friction's description of `dream.cp` ("zips the two ring scans") does not match the file.
- A dream under 4 is sent back and caps the port at 3 (research/reviews/README.md); the port itself reads at 4.
