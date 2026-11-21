// AtCoder DP L Deque: two players take turns removing an end of the row, each
// maximising their own total minus the other's. lead(l, r) is that margin for
// whoever moves on a[l..=r].
let n: 1..=3000 = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()

@memo
fn lead(l: I, r: I) -> Int =
    if l >= r: a[l] else: max(a[l] - lead(l + 1, r), a[r] - lead(l, r - 1))
print(lead(I.first, I.last))
