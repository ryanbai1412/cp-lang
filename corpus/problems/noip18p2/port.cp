// NOIP 2018 Currency System: the smallest set of coins that can pay exactly
// the same amounts. A coin is needed iff smaller coins cannot make it.
const TOP = 25000
domain Amount = 0..=TOP

@!multitest
fn solve():
    let n: 1..=100 = read()
    let coins: [n]1..=TOP = read()
    var payable: [Amount]Bool = false
    payable[Amount(0)] = true
    var needed = 0
    for c in coins.sorted():
        if not payable[Amount(c)]:
            needed += 1
            for a in Amount:
                if let b = a - c and payable[b]: payable[a] = true
    print(needed)
