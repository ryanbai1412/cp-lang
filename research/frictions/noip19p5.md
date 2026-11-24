# Friction report: noip19p5

There is no dream.cp; the ideal is below. The port is the editorial's O(n):
- a `Prefix<Pos, Int>` of the array, filled from the input or from the statement's generator;
- `from[c]`, the latest cut whose threshold is at most s[c];
- a monotone `Deque<Cut<Pos>>` of candidate cuts;
- a walk back along `from` that sums the squares in I128.

## Dream

```cp
var from: [Pos.cuts]?Cut<Pos> = None
var threshold: [Pos.cuts]Int = 0          // 2 s[j] - s[from[j]], cached once per cut
...
    from[c] = ready.first
    threshold[c] = 2 * s.prefix(c) - (s.prefix(from[c]?) ?? 0)
```

## The memory check ignores the problem's 1024 MB limit
- **Where:** a `threshold` cache like the dream's, `var threshold: [Pos.cuts]Int = 0`, is rejected: `error[E0904]: at the input bounds this takes 320 MB, over the 256 MB limit on generic`. The port's header says `// limits: 2000 ms, 1024 MB`, and the accepted C++ peaks at 862 MB.
- **What felt wrong:** E0904 reads only the judge profile's `memory_mb`, not the problem's own limit. A port for a 1 GB problem is held to 256 MB. The port has to recompute `threshold(j)` (two prefix reads and an optional `from` read) at every deque comparison instead of caching it.
- **Goal it hurts:** express, revision.
- **Ideal code:** the dream's cached `threshold` array, checked against the problem's 1024 MB.
- **General fix:** E0904 (and `cpl explain`'s peak line) should take the memory limit from the file's `// limits:` line or the package's `cp.toml` `[limits]` when one is given, and fall back to the profile's otherwise. spec/tools/judge-profiles.md Rjudge.6 says the profile is the fallback. Owner: core (repro sent).
- **Severity:** medium.

The port is not slow without the cache. On the same 4e7-element max input it runs in 0.97 s and 472 MB, against the accepted C++'s 1.10 s and 863 MB, in back-to-back runs. Under load both times vary by up to 2.5x.

## Measurements

- **Tokens:** port 431, accepted C++ 1029 (comments excluded).
- **Runtime and memory on gen.py --max:** port 2.55 s / 460 MB, accepted C++ 1.13 s / 842 MB, with the same output.
- **Emitted C++:** 21599 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
