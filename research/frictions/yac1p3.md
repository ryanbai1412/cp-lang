# Friction report: yac1p3

There is no dream.cp; the ideal is the port. The strategy is point symmetry: move first on the centre when N is odd, otherwise move second, and mirror every judge move. `their_move` returns `?(1..=n, 1..=n)`, with `0 0` as `None`, and `while let` drives each game.

## Dream

The port as written.

There are no dream/port gaps. The judge's `0 0`/`-1 -1` terminators are the protocol's encoding, decoded once at the read.

## Measurements

- **Tokens:** port 135, accepted C++ 883 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 3 MB, accepted C++ 0.22 s / 3 MB, with the same output.
- **Emitted C++:** 9393 bytes.
