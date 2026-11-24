# aac5p2 friction

The port's code is `dream.cp`, so there is no dream/port gap. `ask("?", g, g.label + 1)`
builds the product table in one line, `I.at(...)` keeps an out-of-range quotient out of `I`,
and `?[I]I` says that a try can fail. There are no gaps.

## Measurements
- Tokens: port 214, original 569.
- `--max` port run: 0.008 s / 4 MB. The accepted C++ is interactive and aborts without
  `interactor.py`, so this standalone run has no C++ time to compare.
- Emitted C++: 14,360 bytes.
