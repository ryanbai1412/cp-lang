// Source: IOI 2025 Day 1 (Souvenirs), https://github.com/ioi/task-archive/tree/master/2025/day1/souvenirs
// limits: 1000 ms, 2048 MB
// Problem: interactive (function task). N souvenir types have strictly falling prices
//   P[0] > ... > P[N-1] >= 1, only P[0] known. transaction(M), for P[0] > M >= P[N-1],
//   buys greedily from type 0 (one of each type M still covers) and returns the types
//   bought and the change. Buy exactly i souvenirs of type i in at most 5000 transactions.
// Solution: a "lead" is the first type a transaction buys; its cost (M minus the change)
//   and its bought types give an equation. Walk leads forward from type 0 paying the lead's
//   average (one less for a single type), which buys a later lead, until type N-1 leads.
//   Then from the right peel solved types off each equation; a lead left alone is solved,
//   one with more types pays its average again. No type is bought more than i times on
//   the way, so top each type up to i at its now known price.
// Interaction (interactor.py, the official manager's protocol over stdio): read N and P[0];
//   transaction(M) prints "? M" and reads the k types bought and the change; "!" ends.
// original: corpus/problems/ioi25_souvenirs/orig.cpp (the official model solution plus the sample grader)
// Bounds: n: 2..=100; p0: 1..=1e15; k: 1..=n; bought: [k]0..n

let n: 2..=100 = read()
domain T = 0..n
let p0: 1..=1e15 = read()

fn transaction(m: Int) -> (List<T>, Int):
    print("?", m)
    let k: 1..=n = read()
    let bought: List<T> = read<[k]T>()
    let change: Int = read()
    (bought, change)

var counters: [T]Int = 0
var lead_cost: [T]Int = -1
var lead_bought: [T]List<T> = []

fn buy_next_lead(i: T) -> T:
    let s = lead_bought[i].len
    var m = lead_cost[i] / s
    if s == 1: m -= 1
    let (got, change) = transaction(m)
    for j in got: counters[j] += 1
    let l = got[0]
    lead_cost[l] = m - change
    lead_bought[l] = got
    l

fn buy_souvenirs(p0: Int):
    let last = T.last!
    lead_cost[T.first!] = p0
    lead_bought[T.first!] = [T.first!]
    var lead = T.first!
    while lead < last: lead = buy_next_lead(lead)
    var prices = last
    while prices > T.first!:
        if lead_cost[lead] != -1:
            while lead_bought[lead].len > 1 and lead_cost[lead_bought[lead].last!] != -1:
                lead_cost[lead] -= lead_cost[lead_bought[lead].last!]
                lead_bought[lead].pop()
            if lead == prices:
                prices = prices.prev!
                lead = lead.prev!
            else:
                lead = buy_next_lead(lead)
        else:
            lead = lead.prev!
    for i in T:
        repeat i.ord - counters[i]: transaction(lead_cost[i])

buy_souvenirs(p0)
print("!")
