# yac7p5 review

Score: 3/5
Dream: 3/5

Cold review: already close to bar; `mid` is now one `let` (middle, or next to the elevator for
small ranges) and the split of the eggs is std `partition` over all but the spare. The port now states DMOJ's
7000 ms limit, so the --max case is judged at the real limit.

## Cold review (HEAD)

Port 3/5 (no dream.cp), dream 3/5. A recursive halving, `place(lo, hi, eggs)`, that partitions by one question per egg and visits the nearer side first (lines 24-38), as in the editorial.
- `breaks` writes the global `standing` (lines 16-21), so `place` reads elevator state that a question changed out of sight. Returning it, or keeping it in `place`'s arguments, would make the "nearer side first" rule visible.
- `eggs[..eggs.len - 1]` with `eggs.last!` (lines 30-31) is a `split_last` (Owner: stdlib, in the friction). `cpl explain` also shows the slice copied (`[copy] 31:26`).
- `floor_of: [Egg]Int = 0` (line 15) uses `0` as a placeholder (Owner: idioms, in the friction).
