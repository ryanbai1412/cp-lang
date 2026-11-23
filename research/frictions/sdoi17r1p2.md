# Friction Report: sdoi17r1p2

## 1. `PreferredPaths.access` doesn't yield the cut below `v` (`stdlib`)

- **Where:** `access(v)` sets `rc[v] = None` on its first step, so the path that `v` stopped
  preferring is never yielded; only switches with a `gained` path are. The doc says counting "how
  many paths lie between `v` and the root reads off the switches", which is then wrong: painting
  6 then 4 (tests/08.in) misses the `+1` on the subtree of 4's old preferred child.
- **Repro:** `domain V = 0..3; let t = Rooted.from_parents([None, Some(0), Some(1)]); var pp = PreferredPaths(t); pp.access(2).count(); pp.access(1).collect()` yields `[]`, but 1 stopped preferring 2.
- **Goal it hurts:** hard to get wrong (the obvious loop gives wrong answers, no trap).
- **General fix:** yield the first cut too, e.g. `(lost: ?V, gained: ?V)` with `gained: None` on
  the step at `v` (or a `cut:` field), and fix the doc's claim and the brute test that mirrors it.
- **Workaround:** spec.cp keeps `below: [V]?V`, each vertex's preferred child, from the switches.
- **Severity:** high. **Status:** open (stdlib).

## 2. An enum with domain-element payload fields doesn't read by tag (`compiler`, already filed)

- **Where:** `enum Op = @tag(1) Paint(x: V) | ...` reads the tag as the sum domain's ordinal:
  `cpl run` traps "1 is not in `Op`", and with two payload variants every op silently reads as
  the first variant. Same root as noi02p1 (compiler worklog, 2026-11-22).
- **Workaround:** payloads are `1..=n`, converted with `V(x)`.
- **Severity:** medium. **Status:** open (compiler).
