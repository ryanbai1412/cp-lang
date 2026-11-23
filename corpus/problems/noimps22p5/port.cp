// Source: NOI 2022 Multi-Provincial Team Selection P5 (Bracket),
//   https://dmoj.ca/problem/noimps22p5
// Problem: a balanced bracket string of length 2n (2 <= n <= 4e5), weights 1 <= w_i <= 1e7 on the opening
//   brackets, and x, y in {0, 1}. Turning p(A)(B)q into p(A()B)q costs x w_A + y w_B; pABq -> pBAq
//   is free (weights move along). Print the least cost to leave no substring (A)(B).
// Solution: the end is a chain, one bracket per depth, and free swaps let any bracket of a level
//   host another. Sweep depths down with a bag (Set of (weight, serial)) of the level's brackets:
//   one stays, the rest sink one level at x * host + y * itself. (0, 1): the heaviest stays, pay
//   the rest; (1, 1): the lightest hosts all, the heaviest stays; (1, 0): the lightest hosts all
//   but the second lightest, which stays and hosts it, and at size 2 one sweep keeps the heavier,
//   one the lighter, the cheaper wins; (0, 0): every move is free, 0. O(n log n).
// original: corpus/submissions/5448138.cpp
// The sorted bag is a Set of (value, serial) pairs: the prototype has no SortedBag.
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

// One sweep down the levels: each level's weights join the bag, then one
// leaves. Which one, and what the level costs, depends on the cost mode.
fn sweep(mode: Int, prefer_min: Bool) -> Int:
    var bag = Set<(Int, Int)>()
    var serial = 0
    var sum = 0
    var cost = 0
    for d in 0..n:
        for v in level[d]:
            bag.insert((v, serial))
            serial += 1
            sum += v
        let size = bag.len
        let least = bag.min()!
        let most = bag.max()!
        if mode == 1:
            // x = 0, y = 1: pay everything that stays
            bag.remove(most)
            sum -= most.0
            cost += sum
        elif mode == 3:
            // x = 1, y = 1: the smallest escorts every swap
            cost += sum + (size - 2) * least.0
            bag.remove(most)
            sum -= most.0
        else:
            // x = 1, y = 0
            if size == 1:
                bag.remove(least)
            elif size == 2:
                let gone = if prefer_min or d == n - 2: least else: most
                cost += gone.0
                bag.remove(gone)
            else:
                bag.remove(least)
                let second = bag.min()!
                bag.remove(second)
                bag.insert(least)
                cost += (size - 2) * least.0 + second.0
    cost

if x == 0 and y == 1: print(sweep(1, false))
elif x == 1 and y == 1: print(sweep(3, false))
elif x == 1 and y == 0: print(min(sweep(2, false), sweep(2, true)))
else: print(0)
