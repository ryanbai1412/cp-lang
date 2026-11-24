# aac5p2

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp` (the same code bar the header lines), without the C++.
The N - 1 neighbour products are one `ask` comprehension over `Gap`, `unfold(first) -> ?[I]I` tries
one first value, and `I.at(...)` keeps an out-of-range quotient out of `I`.

## Checklist failures
- idiomatic-code/R3: the driver (lines 31-34) is a search written as a loop with `print` and
  `break`. `print("!", I.first_some(unfold)!)` (std `Seq.first_some`) says "the first try that
  works"; `unfold` already rejects a `first` that does not divide the first product.
- idiomatic-code/R6: `a[I(g.label)]` and `I(g.label + 1)` (lines 22, 26) cross from `Gap` to `I`
  through raw labels. Minor, and the header defines `Gap` as "between g and g + 1".

## Owed rewrites
- The `I.first_some(unfold)` driver above.

## Language gaps
None.

## Dream notes
The dream is the port; it reads like the editorial apart from the search loop above.
