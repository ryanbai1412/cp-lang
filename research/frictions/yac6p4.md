# Friction report: yac6p4

`dream.cp` is verified. The port read the sides into an array and XORed them in a `var need` loop. It now takes the dream's one-line fold, `sides.map(side => grundy[side]).fold(^)`. The fold keeps a named `sides` read so that the `// Bounds:` header names it. Check and lint are clean, and the port agrees with the v0.2 port on 30 stress cases and passes its 8 package cases.

## Dream

`dream.cp`.

## A 101-entry constant table takes 101 lines
- **Where:** lines 15-117, the `grundy` literal, one number per line in both port and dream.
- **What felt wrong:** a table of precomputed constants pushes the four lines of program below the fold. Ten or so numbers per line would read as a table.
- **Goal it hurts:** revision.
- **Ideal code:** the same list, packed to the line width.
- **General fix:** `cpl fmt` fills short scalar list literals to the line width instead of one element per line. Owner: consumers (fmt).
- **Severity:** low.

There are no other dream/port gaps.

## Measurements

- **Tokens:** port 284, accepted C++ 442 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 3 MB, accepted C++ 0.02 s / 7 MB, with the same output.
- **Emitted C++:** 11887 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.

## Addendum: the table's type drops its index range
- **Where:** `let grundy: List<Int>` indexed by `side: 1..=100`.
- **What felt wrong:** `spec.cp` declares the same table as `const grundy: [0..=100]Int`. As a
  `List`, every lookup is a bounds check that the type could have proved.
- **Ideal code:** `const grundy: [0..=100]Int = [...]`.
- **General fix:** none; port choice. Owner: idioms.
- **Severity:** low.
