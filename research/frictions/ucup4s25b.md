# ucup4s25b friction

The port is the dream.

## A type alias of a domain's bounds has no static members

- **Goal it hurts:** express
- **Severity:** low
- **Area:** core
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup4s25b.cp:13,21,38`
- **Actual:** `type Cut = Ball.bounds` names the cut space for annotations, but `Cut.nth(k)` and `Cut.last`
  fail ("`Cut` is a type, and a value is needed here"), so values are spelled `Ball.bounds.nth(k)`.
- **General fix:** an alias of a domain (or of `D.bounds`) carries the domain's static members, so one name
  serves as type and constructor, as `domain` does.

## No read of a character into an enum

- **Goal it hurts:** express
- **Severity:** low
- **Area:** core
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup4s25b.cp:14-15`
- **Actual:** the side is read as `Char` and mapped to `Dir` by hand.
- **General fix:** enums with input spellings (`enum Dir: Left = 'L', Right = 'R'`) read and print directly,
  the shape of every L/R, U/D or +/- token in a statement.

## Measurement

`n` and `q` stay `Int` (the statement's bounds are not recorded); weights are `1..=1e9` from the generator.
n = q = 300,000, `g++ -O2`: 0.09-0.14 s vs 0.14-0.18 s, 111 MB vs 100 MB, same output. Emitted C++
13,555 -> 16,474 bytes (`Prefix`, the merge list and the enum).
