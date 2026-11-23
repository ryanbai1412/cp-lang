// CF 1931F Chat Screenshots: each screenshot orders all but its author; consistent iff acyclic.
@!multitest
let n: 1..=2e5 = read()
let k: 1..=2e5 = read()
domain V = 1..=n
let shots = read<[k][n]V>()
var after: [V]List<V> = [V] _ => []
for row in shots:
    for (a, b) in row[1..].adjacent:
        after[a].push(b)
print(yes(topo(after) != None))
