// expect: E0305
// rule: 07.4.5
// A run walked with its index keeps the parent's keys too; `r[0]` is not the run's first element (decision 388).
let a = [3, 3, 1, 2, 2]
for (k, r) in a.sorted().runs().items:
    print(k, r[0]) // <- here
