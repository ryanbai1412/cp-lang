# Friction report: yac6p2

There is no dream.cp. The port is the ideal:
- both trees are read as `[V]?V` with `read(none: -1)`;
- the roots are compared with `V.find`;
- the workers whose boss differs are moved in `Rooted.from_parents(want)` preorder.

## Dream

The port as written.

There are no dream/port gaps. Any valid move order is accepted (`checker.py`), so the `--max` output differs from the accepted C++ in order only. The checker accepts both, and the port passes its package cases.

## Measurements

- **Tokens:** port 144, accepted C++ 398 (comments excluded).
- **Runtime and memory on gen.py --max:** port 2.33 s / 120 MB, accepted C++ 1.92 s / 293 MB.
- **Emitted C++:** 19599 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
