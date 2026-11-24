// Source: NOI 2022 Multi-Provincial Team Selection P5 (Bracket),
//   https://dmoj.ca/problem/noimps22p5
// limits: 1000 ms, 512 MB
// Problem: a balanced bracket string of length 2n (2 <= n <= 4e5), weights 1 <= w_i <= 1e7 on the opening
//   brackets, and x, y in {0, 1}. Turning p(A)(B)q into p(A()B)q costs x w_A + y w_B; pABq -> pBAq
//   is free (weights move along). Print the least cost to leave no substring (A)(B).
// Solution: the end is a chain, one bracket per depth, and free swaps let any bracket of a level
//   host another. Sweep depths down with a heap of the level's brackets:
//   one stays, the rest sink one level at x * host + y * itself. (0, 1): the heaviest stays, pay
//   the rest; (1, 1): the lightest hosts all, the heaviest stays; (1, 0): the lightest hosts all
//   but the second lightest, which stays and hosts it, and at size 2 one sweep keeps the heavier,
//   one the lighter, the cheaper wins; (0, 0): every move is free, 0. O(n log n).
// original: corpus/submissions/5448138.cpp
let n: 2..=4e5 = read()
let x: 0..=1 = read()
let y: 0..=1 = read()
let s: [domain S]Char = read()
let w: [0..n]1..=1e7 = read()

// level[d]: weights of the brackets nested d deep
var level: List<List<Int>> = [[] for _ in 0..n]
var depth = 0
var opened = 0
for i in S:
    if s[i] == '(':
        level[depth].push(w[opened])
        opened += 1
        depth += 1
    else:
        depth -= 1

// One sweep down the levels: each level's weights join the bag, then one leaves.
// x = 0, y = 1 pays for everything that stays; x = 1, y = 1 also pays the lightest once per
// sunk bracket. Only the heaviest ever leaves and the bag never empties before the last level,
// so the lightest in the bag is the lightest seen.
fn sweep_heavy(escort: Bool) -> Int:
    var bag = MaxHeap<Int>()
    var least = w.max()!
    var sum = 0
    var cost = 0
    for d in 0..n:
        for v in level[d]:
            bag.push(v)
            least min= v
            sum += v
        if escort: cost += sum + (bag.len - 2) * least
        sum -= bag.pop()!
        if not escort: cost += sum
    cost

// x = 1, y = 0: the lightest hosts all but the second lightest, which stays and hosts it; at
// size 2 the sweep keeps the heavier (prefer_min) or the lighter one.
fn sweep_light(prefer_min: Bool) -> Int:
    var bag = MinHeap<Int>()
    var cost = 0
    for d in 0..n:
        for v in level[d]: bag.push(v)
        let least = bag.pop()!
        if bag.len == 1 and (prefer_min or d == n - 2):
            cost += least
        elif not bag.is_empty:
            cost += (bag.len - 1) * least + bag.pop()!
            bag.push(least)
    cost

if x == 0 and y == 1: print(sweep_heavy(false))
elif x == 1 and y == 1: print(sweep_heavy(true))
elif x == 1 and y == 0: print(min(sweep_light(false), sweep_light(true)))
else: print(0)
