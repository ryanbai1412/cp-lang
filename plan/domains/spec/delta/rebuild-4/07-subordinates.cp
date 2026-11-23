// CSES 1674 Subordinates
// in: n, then the boss of each employee 2..n; out: the number of subordinates of each employee 1..n
let n = read<Int>()
domain Emp = 1..=n
var boss: [Emp]?Emp = None
for e in Emp:
    if e != Emp.first!: boss[e] = read<Emp>()
let t = Rooted.from_parents(boss)
print([Emp] e => t.size(e) - 1)
