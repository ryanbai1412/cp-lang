// Source: An Animal Contest 5 P2, https://dmoj.ca/problem/aac5p2
// Problem: interactive. A hidden permutation a of 1..=N (3 <= N <= 1e5); "? i j" answers
//   a_i * a_j, at most N - 1 queries. Print "! a_1 ... a_N".
// Solution: ask the N - 1 neighbour products a_g a_(g+1). Then try each first value dividing the
//   first product: dividing along the chain fixes every value, and the first try that stays
//   a permutation is printed. O(N) per try.
let n: 2..=1e5 = read()
domain I = 1..=n
domain Gap = 1..n  // between positions g and g + 1

// the products of neighbours pin everything down once one value is known
var product: [Gap]Int = 0
for g in Gap:
    let p: 1..=n * n = ask("?", g, g.label + 1)
    product[g] = p

// the permutation that starts with `first`, if there is one
fn unfold(first: Int) -> ?[I]Int:
    var a: [I]Int = first
    var used: [I]Bool = false
    used[I(first)] = true
    for g in Gap:
        let here = a[I(g.label)]
        if product[g] % here != 0: return None
        let next = product[g] / here
        if next not in I or used[I(next)]: return None
        used[I(next)] = true
        a[I(g.label + 1)] = next
    a

for first in 1..=n:
    if product[Gap.first!] % first == 0:
        if let a = unfold(first):
            print("!", a)
            break
