// Source: NOI 2006 P1 (Network Charges), https://dmoj.ca/problem/noi06p1
// limits: 1400 ms, 256 MB
// Problem: 2^N <= 1024 users are the leaves of a full binary tree; each registered paying A
//   or B, and switching user i costs C_i. Each pair i < j is charged at their LCA P: if P
//   has fewer A than B leaves, F_ij per A in the pair, else F_ij per B in the pair. Given
//   the types, C_i <= 5e5 and F_ij <= 500, print the least total of switches plus charges.
// Solution: a pair's charge splits into F_ij per leaf of the charged type at P, so fix at
//   each router a flag (more B than A below it or not) and charge each leaf alone: per
//   ancestor whose flag differs from its type, its flow to that ancestor's other half (a
//   prefix-sum fold). best(u, flags) gives, per count of B under heap node u, the cheapest
//   cost of u's subtree given its ancestors' flags; a router tries both flags, combines its
//   children's tables and keeps the counts that agree with the flag. O(4^N N) in all.
// original: corpus/submissions/5032743.cpp
// Bounds: m: 1..=10; plan: 0..=1; switch: 0..=500_000; f: 0..=500
let m: 1..=10 = read()
let n = 1 << m
domain User = 0..n
let plan: [User]0..=1 = read()  // 1 is B
let switch: [User]0..=500_000 = read()
var flow: [User, User]Int = 0  // i < j only
for i in User:
    for j in User.after(i):
        let f: 0..=500 = read()
        flow[i, j] = f
let reach = Prefix<User, User, Int>([User, User] (i, j) => flow[min(i, j), max(i, j)])

// Heap nodes: the root is 1, node v has children 2v and 2v + 1, user i is leaf n + i.
fn users(v: Int, depth: Int) -> Range<User>:
    let size = 1 << (m - depth)
    let lo = v * size - n
    User(lo)..=User(lo + size - 1)

// cost of u's subtree per count of B under it; bit d of flags is the flag of u's ancestor at depth d
fn best(u: Int, depth: Int, flags: Int) -> List<?Int>:
    if depth == m:
        let i = User(u - n)
        return [0, 1].map(
            b => Some(
                (if plan[i] != b: switch[i] else: 0) + sum(
                    reach.fold(
                        i..=i,
                        users(u >> (m - d - 1) ^ 1, d + 1),
                    ) for d in 0..m if flags.bit(d) != (b == 1)
                )
            )
        )
    let half = 1 << (m - depth - 1)
    let unseen: ?Int = None
    var out = [unseen] * (2 * half + 1)
    for more_b in [false, true]:
        let below = flags | (Int(more_b) << depth)
        let (l, r) = (best(2 * u, depth + 1, below), best(2 * u + 1, depth + 1, below))
        for a in 0..=half:
            for b in 0..=half if (a + b > half) == more_b:
                out[a + b] min= l[a] + r[b]
    out

print(best(1, 0, 0).filter_map(c => c).min()!)
