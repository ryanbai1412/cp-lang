# ioi05p2

Score: 3/5
Dream: 3/5

Re-ported in v0.3 style from spec.cp/dream (pieces + lazy SegTree + right_while).

## Cold review (HEAD)

Port 3/5 (provisional: the dream is under 4), dream 3/5. The port is the dream: `Rail.range.pieces(cuts:)`, a lazy `SegTree` of `Rise` under assign-slope, and `right_while` for the ride (lines 28-59).
- Lines 50-52 write the match arms as `Set(a, b, d):` and `Ask(h):`; `=>` is the one arm spelling (handbook/language/overview.md). `cpl check` and `cpl lint` accept the colon form silently; that gap is now in the friction.
- The command loop (lines 19-26) is in the friction (io tags on enum variants).
- The measurement in the friction is `cpl run` (1.17 s vs 0.058 s), not the built binary, so the 1.1x bar (idiomatic-code/R10) is unmeasured.

## Dream notes

- dream.cp uses colon arms throughout (lines 15, 38-39), so it mixes spellings and caps at 3; with `=>` arms it is a 4.
