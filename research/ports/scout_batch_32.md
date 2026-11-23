# Scout Batch 32: binary search, sweep-line, compression and interval classics

Ten previously unported corpus problems, each solved twice:

- `<problem>.cp` is the ideal port against the locked v2 spec
  (`docs/one-page-cheat-sheet.md`). These files are not executed; they are the
  evidence for `research/archive/frictions/friction-backlog-batch-32-bsearch-sweep.md`, and every
  rough spot is marked in place with a `WISH X<n>` comment.
- `today/<problem>.cp` is a runnable version for the reference interpreter
  (`python3 proto/cp.py lib-run`). It uses only what `proto/cpi` and `stdlib/`
  implement today, so searches, skip pointers and compression are written out
  by hand there.

## Problems

- `ioi04p2hard` (4182973): IOI 2004 Hermes with large limits. DP whose
  transitions are "cheapest offer by distance", kept in ordered maps of
  prefix minima and queried with `last_le`.
- `rpeng3` (4811952): IOI 2009 Salesman. Sweep over days with two max segment
  trees over positions and a two-pass DP inside each day.
- `ceoi22p5` (4733784): CEOI 2022 Measures. Offline rank compression of all
  arrivals and a segment tree with a four-field merge.
- `996warmup8p3` (5324670): bouncing balls. Binary search on time over a closed
  form simulation, then a sort of the balls still heading for the unfinished wall.
- `iati21a22` (5053473): rotate coordinates, then a strict LIS; the C++
  compresses both axes, the port shows it is not needed.
- `aac5p3` (4239862): values routed down a weighted tree; each vertex splits its
  value interval at midpoints between sorted child weights and counts with two
  rank lookups.
- `ucrpc21e` (3944571): event sweep over the endpoints of two teams' scoring
  intervals.
- `noimps23p1` (5556659): interval reachability; every position is swept once
  out of an ordered set of unswept positions.
- `noip10p1` (5445654): NOIP 2010 Machine Translation. FIFO window of the last
  `m` distinct words.
- `dmopc20c4p4` (3485716): online prefix costs with a stack of set-aside
  elements.

The corpus has no problem statements, so each header describes the task as
reconstructed from the accepted C++; input bounds in the ideal ports are the
published ones where the problem is a well-known classic and plausible
placeholders otherwise.

## Verification

```text
$ python3 proto/ports/scout_batch_32/check.py
[PASS] ioi04p2hard  OK (40/40 cases match 4182973.cpp)
[PASS] rpeng3       OK (40/40 cases match 4811952.cpp)
[PASS] ceoi22p5     OK (40/40 cases match 4733784.cpp)
[PASS] iati21a22    OK (40/40 cases match 5053473.cpp)
[PASS] aac5p3       OK (40/40 cases match 4239862.cpp)
[PASS] ucrpc21e     OK (40/40 cases match 3944571.cpp)
[PASS] noimps23p1   OK (40/40 cases match 5556659.cpp)
[PASS] noip10p1     OK (40/40 cases match 5445654.cpp)
[PASS] dmopc20c4p4  OK (40/40 cases match 3485716.cpp)
[PASS] 996warmup8p3 OK (40/40 cases match 5324670.cpp)
```

`check.py` compiles each original with `g++ -std=c++17 -O2` and compares
whitespace-separated tokens against `lib-run` on edge cases and seeded random
inputs (three quarters small, one quarter medium, up to a few hundred
elements). `check.py <problem> <cases>` runs one port with more cases.

Two generators restrict the input, and the reason is the reference, not the port:

- `aac5p3` uses distinct edge weights, because the C++ splits equal weights at
  one vertex through an unstable `std::sort`.
- `dmopc20c4p4` only emits requests the C++ can serve; anything larger pops an
  empty deque there.

What is not tested: the ideal ports (no implementation accepts the full v2
surface yet), and large inputs (the interpreter is too slow for judge sizes).
In the runnable `rpeng3` the two segment trees, and in `ioi04p2hard` the
ordered maps, are plain lists scanned linearly, which is enough for the test sizes.

## Size

Non-blank, non-comment lines. "Stripped" C++ drops includes, the debug
template block, fast-I/O lines and the stock segment-tree struct, as if C++ already had a perfect contest library.

- `996warmup8p3`: C++ 122 raw / 91 stripped, CP Lang 36 (70% / 60% shorter)
- `aac5p3`: 83 / 52, CP Lang 16 (80% / 69%)
- `ceoi22p5`: 109 / 57, CP Lang 24 (77% / 57%)
- `dmopc20c4p4`: 41 / 30, CP Lang 16 (60% / 46%)
- `iati21a22`: 71 / 42, CP Lang 8 (88% / 80%)
- `ioi04p2hard`: 107 / 78, CP Lang 38 (64% / 51%)
- `noimps23p1`: 95 / 71, CP Lang 26 (72% / 63%)
- `noip10p1`: 55 / 27, CP Lang 11 (80% / 59%)
- `rpeng3`: 133 / 66, CP Lang 38 (71% / 42%)
- `ucrpc21e`: 79 / 47, CP Lang 18 (77% / 61%)

Total: 895 raw and 561 stripped C++ lines against 231 lines of CP Lang, 74%
and 58% shorter.
