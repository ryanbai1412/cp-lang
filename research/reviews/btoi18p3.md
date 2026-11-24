# btoi18p3 review

Score: 2/5
Dream: 2/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. The header
explains all three cases: golden-section search on a line, separator-line halving on a sheet,
random sampling plus hill climbing in a box. The line and box cases are short. The sheet case
is written twice.

## Checklist failures
- idiomatic-code/R4: `on_sheet` has two mirrored branches (lines 61-78 and 79-96): the same
  probe, compare and recurse, once with x and y swapped. Unreported.
- idiomatic-code/R7: `height` asks the judge and counts `asked` (lines 24-26), yet it is called
  inside predicates and comparisons: `beside(c).find(d => height(d) > height(c))` (line 104),
  `.max(by: height)` (line 103), `(l..r).find(x => at(x) >= ...)` (line 52). Every query hides
  in an expression.
- idiomatic-code/R5/R6: `type Cell = (Int, Int, Int)` (line 16) drops the box's bounds and
  axes; in the friction (off-box neighbours read 0).

## Owed rewrites
- Write the sheet step once, over the longer axis, with the swap at its boundary.
- Add `cpl run` output to the friction for the claimed `@memo` on an `ask` and the product
  domain whose neighbours stay inside (idiomatic-code/R11).

## Language gaps
- Product-domain neighbours, `@memo` on an interactive `ask`, a `Rect` for the sheet: in the
  friction, without runs.

## Dream notes
The dream is declarations and one signature, `fn on_sheet(r: Rect<X, Y>, seen: ?(Cell, Int))
-> ?Cell`, with no body. It names the right types but not the search. Write the whole program
(send back).
