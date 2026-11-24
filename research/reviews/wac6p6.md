# wac6p6

Score: 3/5
Dream: 3/5

Cold review: three strategies on the sorted counts a <= b <= c, each a short loop over the canes. Colours and bowls are domains, an unassigned colour is a None bowl instead of -1, and the second bowl index is derived from how many colours already hold one. The new interactor checks one-colour bowls and at least 43 kept per factory.

## Cold review (HEAD)

Port 3/5 (no dream.cp), dream 3/5. All three strategy cases from the header are separate `for` loops over `cane()`, and a `[Colour]?Bowl` with `put(bowl[x])` is the right shape.
- `cane()` (lines 20-22) maps a letter to a colour with an `if` chain ending in `Colour(2)`, and `(red, green) = (Colour(0), Colour(1))` (line 18) names ordinals by hand. A `domain Colour = "RGB"` read as a letter would delete both (Owner: core/spec, in the friction).
- Lines 40-42: fmt splits `Bowl(1 + Colour.count(y => bowl[y] != None))` over three lines inside an `if`. Binding `let next = ...` first would read on one line today; the port owes that.
- `sorted[0], sorted[1], sorted[2]` (line 30) unpacks a length-3 array by index (Owner: core, destructuring).
- Interactive; no speed concern.
