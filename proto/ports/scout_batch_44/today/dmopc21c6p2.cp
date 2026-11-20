// Runnable port (proto/cp.py lib-run) of dmopc21c6p2.cp
// original: corpus/submissions/4342045.cpp

let (n, k): (Int, Int) = read()
let s: Str = read()

// Lengths of the maximal runs of '0'; a trailing sentinel closes the last run.
var zero_runs: List<Int> = []
var run = 0
for c in s + "1":
    if c == '0':
        run += 1
    elif run > 0:
        zero_runs.push(run)
        run = 0

let longest = sorted(zero_runs, by: x => -x)
var ans = 0
for i in 0..min(k, longest.len): ans += longest[i]
print(ans)
