# tools/bench

`bench.py` measures every corpus port in `proto/ports/` (and in `proto/ports/hillclimb_dp_graph/`,
the other folder `proto/run_all.sh` checks) against the accepted C++ submission it re-solves. It is the repeatable source of the numbers in `research/ports-report.md`.

```sh
python3 tools/bench/bench.py                                     # every port
python3 tools/bench/bench.py --ports dpz,hillclimb_dp_graph/dpo  # some ports
python3 tools/bench/bench.py --seeds 5 --limit 4                 # more inputs, larger inputs
```

Needs `python3`, `g++` (C++17) and GNU `time` at `/usr/bin/time`, nothing else.

## What it measures

For each port the harness builds `<name>.cp` to C++ with the prototype compiler and compiles
both it and `<name>_orig.cpp` with `g++ -std=c++17 -O2`. Then it records:

- **Correctness.** The port and the original must print the same tokens on every measured
  input. Interactive ports (those with a `<name>_judge.py`) are run against their judge
  instead, and both sides must be accepted.
- **Lines.** Port lines against the original twice: raw, and library-stripped, where the line
  ranges in `<name>.strip` (template, driver, DSU, hashing, ...) are left out as if C++ had a
  perfect contest library. Counting is `proto/loc.py`'s: no blank or comment-only lines.
- **Emitted C++.** Bytes of the generated `.cpp`, and the stripped `-O2` binary relative to
  the original's binary.
- **Runtime and peak RSS.** Wall time and maximum resident set size of both programs, the
  worst over the measured inputs.

## Where inputs come from

Every port describes its input with typed `read()` calls, generators and `where`
preconditions, so the port's own generator (`<name>_gen seed size`) produces valid inputs.
`size` caps the generated ranges. The harness walks the caps `1000000, 300000, ..., 6` and
takes the largest one at which the original still answers within `--limit` seconds (default
2), then measures the first `--seeds` accepted inputs (default 3) at that cap. Seeds are
fixed, so two runs on one machine measure the same inputs.

## Output

One JSON object per port is printed as it finishes. `results.json` (all fields) and
`results.md` (a table with a summary) are written next to the script, or to `--out DIR`.
The exit code is 1 if any port fails to build or disagrees with its original.
