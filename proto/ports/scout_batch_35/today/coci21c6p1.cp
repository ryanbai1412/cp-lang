// Med, in the forms proto/cpc builds today. See ../coci21c6p1.cp.
// original: corpus/submissions/4529987.cpp
let n: Int = read()
var name: List<Str> = []
var total: List<Int> = []
for _ in 0..n:
    let who: Str = read()
    let rounds: [5]Int = read()
    name.push(who)
    total.push(rounds.sum())

fn place(i: Int, mine: Int, theirs: Int) -> Int:
    let me = total[i] + mine
    var ahead = 0
    for j in 0..n:
        let them = total[j] + theirs
        if j != i and (them > me or (them == me and name[j] < name[i])): ahead += 1
    1 + ahead

for i in 0..n: print(place(i, 500, 0), place(i, 0, 500))
