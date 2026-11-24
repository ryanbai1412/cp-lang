# Judge profiles

Status: draft for 0.3.0. Sources: `plan/70-judges.md`, decisions 31, 34,
148. Implemented in `crates/cpl-judge`.

A profile fixes everything that differs between judges: compiler and
flags, OS, the judge's stack and the program thread's stack, source
size limit, `__int128` availability, time and memory limits, `answers`
and `precision` defaults for `io`, instruction set, and which checks
stay in release.

- Built in: `generic` (default), `codeforces`, `atcoder`, `cses`,
  `icpc`, `kattis`, `dmoj`, `usaco`; `ioi` (grader-style) is proposed. (Rjudge.5)
- Chosen by `--judge`, else a `// judge: name` line, else `cp.toml`.
  `[judge.<name>]` tables override fields or `extends` a profile. (Rjudge.1)
- The C++ file's second line names the profile, its compiler command
  and its stack, so the file says how it must be submitted:
  `// For judge `codeforces`: g++ ... -Wl,--stack=268435456; 256 MB stack.`
  `codeforces` is the Milestone 1 judge; local release runs use the same
  command and stack. (Rjudge.2)

```toml
[judge.school]
extends = "icpc"
stack_mb = 64
thread_stack_mb = 512        # 0 if the judge bans threads
source_kb = 100
flags = ["-std=c++17", "-O2"]
int128 = false
time_ms = 2000
memory_mb = 256
release_checks = []          # e.g. ["ranges", "overflow", "reads"]
isa = "x86-64"               # "avx2" or "unknown": target pragmas (Rjudge.9)
answers = ["YES", "NO"]
```

- `I128` on a profile without `__int128` is an error naming the judge (Rjudge.3).
- Size: a warning above 32 KB (or half of a smaller `source_kb`), an
  error above 64 KB or `source_kb`, whichever is smaller; `--size`
  breaks the file down by part (decision 188) (Rjudge.4).
- `time_ms` and `memory_mb` are a problem's limits when its `cp.toml`
  has no `[limits]`. (Rjudge.6)
- `memory_mb` is the limit of the peak-memory error E0904 (R03.1.9), and
  feeds `cpl explain`'s peak line and the LSP's code lens; `time_ms`
  feeds `cpl bench`. (Rjudge.7)
- Local release runs build with the profile's flags and run under its
  stack (`ulimit -s`), so judge-only crashes reproduce locally. (Rjudge.8)
- `isa = "avx2"` says the judge's CPUs have AVX2. The C++'s first pragma
  then asks for `optimize("O3,unroll-loops")` as well as no FMA, since -O2
  alone never vectorises a loop whose length is known only at run time,
  and `#pragma GCC target("avx2,bmi,bmi2,lzcnt,popcnt")` follows the
  includes: x86-64-v3's features, so `popcount`, `clz` and `ctz` are one
  instruction each. `x86-64`, the default, emits no `optimize` pragma and
  `#pragma GCC target("popcnt")`, since every x86-64 judge CPU since
  Nehalem has it; `unknown` (CPUs not known, or not x86) emits neither.
  Each `isa` and its features are one row of cpl-judge's `ISAS`
  (decision 586). (Rjudge.9)
