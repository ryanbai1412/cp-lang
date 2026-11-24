// Source: IOI 2008 Day 1 (Fish), https://dmoj.ca/problem/ioi08p3
// limits: 1400 ms, 64 MB
// Problem: F <= 5e5 fish, each with a length <= 1e9 and one gem of kind 1..=K; a fish can eat
//   any fish at most half its length (keeping its gems, its length unchanged). Count the distinct
//   per-kind gem counts that can end up in one fish, mod M (2 <= M <= 30000).
// Solution: a fish of length L can hold its own gem plus any subset of fish <= L/2. Rank kinds by
//   their longest fish and credit each combination to the kind whose longest fish makes it. For
//   kind r, a two-pointer adds fish <= max[r]/2 to per-kind counts c, a SegTree keeps products
//   of (c + 1) mod M. Fewer than all c[r] + 1 of r's gems: c[r] * prod over lower kinds; all of
//   them: also times prod over higher kinds whose longest fish is below twice r's next fish
//   (`first_true` finds the bound). O((F + K) log K).
// original: corpus/submissions/4843416.cpp
// Bounds: fishes: 1..=5e5; kinds: 1..=fishes; m: 2..=30000; fish.len: 1..=1e9; fish.kind: 1..=kinds
let (fishes, kinds, m): (1..=5e5, 1..=fishes, 2..=30000) = read()
type Z = Mod<m>
domain Kind = 1..=kinds
let fish: [fishes](len: 1..=1e9, kind: Kind) = read()

var longest: [Kind]Int = 0
for (len, kind) in fish: longest[kind] max= len
// Kinds ranked by their longest fish; every count is credited to the top-ranked kind it uses.
domain Rank = 0..kinds
let order = Kind.sort(by: g => (longest[g], g))
var rank_of: [Kind]Rank = Rank.first!
for (r, g) in order.items: rank_of[g] = Rank.nth(r)
let top: [Rank]Int = [Rank] r => longest[order[r.ord]]
let eaten = fish.sorted(by: f => f.len)
var lens: [Rank]List<Int> = []  // each kind's fish lengths, increasing
for (len, kind) in eaten: lens[rank_of[kind]].push(len)

var small: [Rank]Int = 0  // fish of each kind at most half the current top fish
var choices = SegTree<Rank, Z>.of(*, _ => Z.one)  // small[q] + 1 per kind
var next = 0
var total: Z = 0
for r in Rank:
    while next < eaten.len and eaten[next].len <= top[r] / 2:
        let q = rank_of[eaten[next].kind]
        small[q] += 1
        choices.set(q, Z(small[q] + 1))
        next += 1
    let lower = choices.fold(..r)
    // Fewer than all of r's gems: 1..=small[r] of them, eaten by r's longest fish.
    total += lower * Z(small[r])
    // All of them: the shortest fish of r above half its top must be eaten too, which frees the
    // higher kinds whose longest fish is under twice it.
    let big = lens[r][small[r]]
    let free = (r.ord + 1..kinds).first_true(q => top[Rank.nth(q)] >= 2 * big) ?? kinds
    total += lower * choices.fold(r.r..Rank.cuts(free))
print(total)
