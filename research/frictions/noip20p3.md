# Friction report: noip20p3

The port is now the body of `dream.cp`, which compiled and was verified, with the statement bounds as read types (`n: 2..=50`, `m: 2..=400`, colours `1..=n`). The v0.2 port was the accepted C++ translated:
- global `g_n`, `g_m`, `g_stks`, `g_mu`, `g_mv` and `g_empty`;
- a `ri()` read helper and 0-based `Int` pillars;
- a header comment working around a nested recursive fn bug that no longer exists.

The new port's tubes are `domain Tube = 1..=n + 1`. Moves are `List<(Tube, Tube)>`, and `split` keeps the dream's nested `low`/`mixed`/`bottoms` helpers.

## Dream

`dream.cp`, which the port now matches. There is no dream/port gap.

## Measurements

- **Tokens:** port 731, accepted C++ 1517 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.02 s / 6 MB, accepted C++ 0.04 s / 7 MB.
- **Emitted C++:** 24438 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
