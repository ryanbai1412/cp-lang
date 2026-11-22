// cp: 0.1 1f59ac0+dirty
// AtCoder DP L Deque: players alternately take an end of the row; each
// maximises (own total - opponent's). lead(l, r) is that margin for whoever
// moves on a[l..=r].
let n: 1..=3000 = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()

@memo
fn lead(l: I, r: I) -> Int:
    if l == r: return a[l]
    max(a[l] - lead(l.next!, r), a[r] - lead(l, r.prev!))
print(lead(I.first!, I.last!))
