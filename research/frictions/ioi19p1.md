# Friction report: ioi19p1

spec.cp wanted a table over a signed anonymous range, which R04.2.25 allows
and `cpl run`/`cpl build` reject; it now names the range as a domain, which
both engines run.

## A table over a signed anonymous range is E0501 in `cpl run` and `cpl build`

- **Severity:** medium
- **Status:** open (routed to compiler, worklog 2026-11-22)
- **Goal it hurts:** express
- **Where:** `var next_of: [-n..=n]Deque<P> = [-n..=n] v => Deque()`,
  indexed by the shoe `x: -n..=n` and its mirror `-x`.
- **What happened:** the plain-range table is the spec's form for keying by
  an `Int` (R04.3.3: arrays over an anonymous range take any `Int`), but both
  `cpl check` and the engines stop with E0501 "can't handle an array dimension
  that starts at anything but a number yet". Repro:
  `let n = read<1..=5>(); var c: [-n..=n]Int = 0; c[-n] += 1; print(c[-n])`.
- **Ideal code:** the line above; `next_of[-x]` needs no conversion.
- **Meanwhile:** `domain Shoe = -n..=n`, `read<[P]Shoe>()` and
  `next_of[Shoe(-Int(x))]` (negating a position takes `Int` first, decision
  379).
- **General fix:** lower an anonymous range axis with a non-literal start as
  an offset by its start, the same as a named domain.
