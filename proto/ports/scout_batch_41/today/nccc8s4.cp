// Mock CCC '21 S4 (nccc8s4), buildable under proto/cpc today. See ../nccc8s4.cp.
// original: corpus/submissions/3381215.java
domain Side = 0..=52
let m: 1..=100_000 = read()
var adj: [Side, Side]Bool = false
for _ in 0..m:
    var tile: [0..4]Int = 0
    for j in 0..4:
        let v: -26..=26 = read()
        tile[j] = v
    for i in 0..4:
        if tile[i] != 0:
            for j in 0..4:
                if j != i and tile[j] != 0:
                    adj[Side.from_idx(tile[i] + 26), Side.from_idx(26 - tile[j])] = true

var state: [Side]Int = 0
var has_cycle = false
fn dfs(u: Side):
    if state[u] != 0:
        if state[u] == 1: has_cycle = true
        return
    state[u] = 1
    for v in Side:
        if adj[u, v]: dfs(v)
    state[u] = 2

for u in Side: dfs(u)
if has_cycle: print(1)
else: print(0)
