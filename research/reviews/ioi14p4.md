# ioi14p4 review

Score: 4/5
Dream: 3/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi14p4.md).

Reads like the editorial for all three tasks: validity as "distinct, one rotation", the
replacement walk where the largest slot absorbs the unused numbers, and the count as a product
of `open ** gap`. The task switch, the `all(s => s == shift!)` agreement test and task 2's many
answers are reported.

## Checklist failures

- `ways` threads `prev` and `open` by hand through the sorted finals (lines 51-56); a
  `with_prev` over `seq.sorted()` would carry `prev`.
- `print(Int(ok))` turns the verdict into 0/1 by hand for the stdin task form.

## Language gaps

- All in research/frictions/ioi14p4.md.

## Dream notes

- Three grader signatures and one `rotation` line; it does not show the replacement walk or
  the count. It should spell `ways` as
  `prod(Z(open_before(g)) ** (g - prev - 1) for (prev, g) in finals.with_prev(start: n))`.
