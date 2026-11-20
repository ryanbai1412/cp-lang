// Educational DP L, Deque: two players alternately take a number from either
// end of a row. Each maximises their own total minus the other's.
// original: corpus/submissions/4131776.cpp
let n: 1..=3000 = read()
domain I = 0..n
let a: [I]1..=1e9 = read()

// lead(l, r): the best margin for the player to move when a[l..=r] is left.
@memo
fn lead(l: I, r: I) -> Int:
    if l == r: return a[l]
    max(a[l] - lead((l + 1)!, r), a[r] - lead(l, (r - 1)!))
print(lead(I.first, I.last))
