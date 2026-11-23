// expect: E0305 at 4
// rule: 07.4.5
// A run keeps its parent's keys; `r[0]` is not "the run's first", `r.nth(0)` is (decision 388).
let a = [1, 1, 2, 2, 2]
for r in a.runs():
    print(r[0])
