// DMOPC '21 Contest 6 P2 — total length of the k longest runs of '0'.
// original: corpus/submissions/4342045.cpp
let (n, k): (Int, Int) = read()
let s: Str = read()

let zero_runs = [len for (c, len) in runs(s) if c == '0']
// FRICTION F5: "k largest" is sort-descending + take; no `desc:` on `sorted`.
print(zero_runs.sorted(by: x => desc(x)).take(k).sum())
