# Friction report: coci25c3p1

Rethought. The v0.2 port kept Ivo as two loose `r`/`c` variables with four copy-pasted `if let x = Col.at(..)` branches, and built each output line by pushing chars. The v0.3 port has one `Cell` and one `step(p, letter) -> ?Cell` that is `None` off the grid, as in egoi23p3. It picks moves with `"LRUD".contains(letter)`, measures distance on the domain values directly and prints a row with `"".join`.

No language gap.
