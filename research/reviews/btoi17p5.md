# btoi17p5

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction)

Cold review of `port.cp`, without the C++. The header gives the whole argument: every row
alternates or every column does, each family is 2^(free lines) once the pinned lines agree, and
the fitting checkerboards are subtracted. The shared `alternating` helper writes the check
once. With no dream the port scores at most 3.

## Checklist failures
- idiomatic-code/R5: the checkerboard call pins line `0` (line 32) into
  `Map<1..=1e9, Bool>` (line 20). `0` is outside the key type and nothing traps. Filed in the
  friction now, with a run.
- idiomatic-code/R1: the comment on lines 17-18 says each pin is "is its line's first cell a
  plus?". The pin is `(f.sign == '+') != odd(f.col)`, which is false for a plus in column 1.
  The count only needs one consistent label, but the stated meaning is wrong.

## Owed rewrites
- Pin the checkerboard on line `1`, or type `pins` with the key type so a bad key is a
  compile error.
- Reword lines 17-18: the pin is the sign the line would have at an even position.
- Write a dream.

## Language gaps
- An `Int` key out of a `Map`'s range key type is accepted silently: added to the friction.
