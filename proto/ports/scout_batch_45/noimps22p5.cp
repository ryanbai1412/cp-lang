// NOI MPS 2022 P5, emptying a bracket forest level by level under two swap costs
// original: corpus/submissions/5448138.cpp
let n: 1..=4e5 = read()
let (x, y): (0..=1, 0..=1) = read()
let s: Str = read()
let w: [0..n]Int = read()

// level[d]: weights of the brackets nested d deep
var level: [0..n]List<Int> = []
var (depth, opened) = (0, 0)
for c in s:
    if c == '(':
        level[depth].push(w[opened])
        opened += 1
        depth += 1
    else:
        depth -= 1

// One sweep down the levels: each level's weights join the bag, then one
// leaves. Which one, and what the level costs, depends on the cost mode.
fn sweep(prefer_min: Bool) -> Int:
    var bag = SortedBag<Int>()
    var (held, cost) = (0, 0)
    for d in 0..n:
        for v in level[d]:
            bag.add(v)
            held += v
        let (size, least, most) = (bag.len, bag.first!, bag.last!)
        let gone = match (x, y):
            (0, _):
                cost += held - most                  // pay everything that stays
                most
            (1, 1):
                cost += held + (size - 2) * least    // the smallest escorts every swap
                most
            (1, 0):
                if size == 1: least
                elif size == 2:
                    let pick = if prefer_min or d == n - 2: least else: most
                    cost += pick
                    pick
                else:
                    let second = bag.nth(1)!
                    cost += (size - 2) * least + second
                    second
        bag.remove(gone)
        held -= gone
    cost

if (x, y) == (1, 0): print(min(sweep(false), sweep(true)))
elif (x, y) != (0, 0): print(sweep(false))
