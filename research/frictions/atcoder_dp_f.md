# atcoder_dp_f

Ported from `corpus/submissions/4131645.cpp`. Dream: `corpus/problems/atcoder_dp_f/dream.cp`.

## The dream's table is wider than the port's
`dream.cp` declares `var common: [I, J]Int`. The port uses `[I, J]0..=3000`, which states the LCS
bound and lets codegen store the 9M-cell table narrow: 19.8 MB against the accepted C++'s
36.9 MB. The port is the better form here, so the dream should take the port's type. That is a
one-token edit to `dream.cp` (owner: idioms, this package), and it lands with the next
`.cp`-edit commit once the `copies` slice is green on main.

## No other gaps
The reconstruction walk (`while common[i, j] > 0`, take equal heads or drop a head that keeps
the length) is the editorial's. `i.next!` is safe because `common[i, j] > 0` means neither
suffix is empty, which the header states.

## Measurements
- Tokens: port 256, original 436.
- `--max` (|s| = |t| = 3000, best of 3): port 31.7 ms / 19.8 MB, original 63.8 ms / 36.9 MB.
  The outputs differ, which is expected: any longest common subsequence is accepted. Both
  have the same length.
- Emitted C++: 12,375 bytes.
