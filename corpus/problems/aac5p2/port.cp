// Source: An Animal Contest 5 P2, https://dmoj.ca/problem/aac5p2
// Problem: interactive. A hidden permutation a of 1..=N (3 <= N <= 1e5); "? i j" answers
//   a_i * a_j, at most N - 1 queries. Print "! a_1 ... a_N".
// Solution: ask the N - 1 neighbour products a_g a_(g+1). Then try each first value dividing the
//   first product: dividing along the chain fixes every value, and the first try that stays
//   a permutation is printed. O(N) per try.
// Bounds: n: 3..=1e5
let n: 3..=1e5 = read()
domain I = 1..=n
domain Gap = 1..n  // between positions g and g + 1

// the products of neighbours pin everything down once one value is known
let product: [Gap]1..=n * n = [Gap] g => ask("?", g, g.label + 1)

// the permutation that starts with `first`, if there is one
fn unfold(first: I) -> ?[I]I:
    var a: [I]I = first
    var used: [I]Bool = false
    used[first] = true
    for g in Gap:
        let here = a[I(g.label)]
        if product[g] % here.label != 0: return None
        if let next = I.at(product[g] / here.label) and not used[next]:
            used[next] = true
            a[I(g.label + 1)] = next
        else:
            return None
    a

for first in I if product[Gap.first!] % first.label == 0:
    if let a = unfold(first):
        print("!", a)
        break
