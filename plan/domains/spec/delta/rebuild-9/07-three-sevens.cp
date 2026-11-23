// CF 1798B Three Sevens
// in: t; per test m, then m blocks "n_i" / "a_i1..a_in_i". out: m winners (one per day), or -1.
@!multitest
let m = read<1..=5e4>()
let days = read<[m]List<1..=5e4>>()

// a day's winner must never play again: their last day is that day
var last: Map<Int, Int> = [:]
for (day, people) in days.items:
    for p in people: last[p] = day

var winners = List<Int>()
for (day, people) in days.items:
    let w = people.find(p => last[p] == day) else:
        print(-1)
        return
    winners.push(w)
print(winners)
