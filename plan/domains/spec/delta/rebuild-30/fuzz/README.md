# Fuzz harness

Python brute forces and generators for round 30. `CPL` names the `cpl` binary,
`BIN` a directory holding the native builds (`NN-<cfid>` from `cpl build` + `g++ -O2`).

- `python3 fuzz.py [name...]`: 200 random inputs per file; brute vs `cpl run` vs native.
- `python3 fuzz_int.py`: 200 hidden tests for 06-1934C; `cpl run` with its `@interactor` vs native with a Python judge (`BIN06` overrides the binary).
- `python3 maxgen.py`: maximum inputs into `max/`.
