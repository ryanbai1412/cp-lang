// SAC '22 CC2 P3, tuned: port.cp's tree DP as one recursion down each fork's two branches, as the
// original does, instead of building a `Rooted` tree from parent links first. Depth k + 1.
let (k, r): (1..=19, 0..=1e9) = read()
domain Fork = 0..2 ** k - 1
domain Node = 1..=2 ** (k + 1) - 1
let forks: [Fork](at: Node, take: Node, skip: Node, gain: -1e9..=1e9) = read()

var fork_at: [Node]?Fork = None
for f in Fork: fork_at[forks[f].at] = f

fn best(v: Node) -> Int:
    let f = fork_at[v] else: return 0
    max(forks[f].gain + best(forks[f].take), best(forks[f].skip))
print(r + best(Node.first!))
