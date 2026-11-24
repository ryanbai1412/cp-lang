# noi21p4

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). The pigeonhole over 16-bit blocks and per-block buckets is the editorial. The friction report says the dictionary is a `[W]Word` and blocks parse with `parse_int` "instead of a hand-written `hex`"; the port keeps `[W, Quarter]U64` (line 38) and a hand `hex` (lines 47-49).

## Checklist failures

- 2.52 s vs C++ 1.28 s at max (2x) is explained but filed as "No friction left" (idiomatic-code/R10); now a friction (section 1).
- The friction report describes a different port.

## Owed rewrites

- Correct the friction report.
- Write the dream.

## Language gaps

- Per-candidate popcount over 16 half-words vs four 64-bit xors: research/frictions/noi21p4.md section 1 (added by this review).
