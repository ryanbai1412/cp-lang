// NOIP 2018 Currency System (proto/ports/noip18p2.cp), with a planted
// multitest leak: the table of payable amounts lives above the
// `@!multitest` divider, so coins from an earlier test still pay.
// class: multitest-leak
const TOP = 25000
domain Amount = 0..=TOP
var payable: [Amount]Bool = false  // planted

fn needed(coins: List<Int>) -> Int:
    payable[Amount(0)] = true
    var needed = 0
    for c in coins.sorted():
        if not payable[Amount(c)]:
            needed += 1
            for a in Amount:
                if let b = a.back(c) and payable[b]: payable[a] = true
    needed

@alt(needed)
fn needed_slow(coins: List<Int>) -> Int:
    var count = 0
    for c in Set(coins):
        var pay = List.filled(c + 1, false)
        pay[0] = true
        for d in coins if d < c:
            for v in d..=c:
                if pay[v - d]: pay[v] = true
        if not pay[c]: count += 1
    count

@!multitest
let n: 1..=100 = read()
let coins: [n]1..=TOP = read()
print(needed([c for c in coins]))
