# egoi22p4

Score: 3/5
Dream: none

Cold review of `port.cp` (no `dream.cp`, no friction `## Dream`, so capped at 3). Runs are a std `IntervalMap` of `(city, since)` and `on_old` settles each overwritten run into a `DualFenwick` (lines 28-37): the header's argument with no sentinels.

## Checklist failures
None found.

## Owed rewrites
- Write the dream; the port could serve as it.

## Language gaps
- Speed: 0.19 s vs 0.13 s at `--max` (1.46x), against a C++ that needs an unlimited stack; measured in the friction, routed to no owner.

## Dream notes
No dream.
