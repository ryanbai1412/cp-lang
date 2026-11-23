// Runnable port (proto/cp.py lib-run) of mccc3j4.cp
// original: corpus/submissions/4223440.cpp

let (n, t): (Int, Int) = read()
domain Player = 1..=n
let init: [Player]Player = read()

var q: [Player]List<Player> = [Player] p => [init[p]]
var head: [Player]Int = [Player] _ => 0
var last: [Player]?Player = [Player] _ => None

for _ in 0..t:
    var arrivals: List<(to: Player, from: Player)> = []
    for p in Player:
        if head[p] < q[p].len:
            let target = q[p][head[p]]
            head[p] += 1
            last[p] = target
            arrivals.push((to: target, from: p))
    for item in arrivals:
        q[item.to].push(item.from)

var line = ""
for p in Player:
    if p.to_idx > 0: line += " "
    line += "{last[p]!}"
print(line)
