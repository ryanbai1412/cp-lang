// Codeforces 2129 B — Stay or Mirror
//
// Each `p[i]` may stay `p[i]` or mirror to `2n - p[i]`. For any pair `(i, j)`
// with `p[j] > p[i]`, both `p[j]` and `2n - p[j]` lie strictly between `p[i]`
// and `2n - p[i]`, so the inversion between `i` and `j` depends solely on
// whether `p[i]` stays (inverting with larger elements to its left) or mirrors
// (inverting with larger elements to its right). Each `i` therefore chooses
// independently `min(left_larger, right_larger)`. Runs in O(n^2) (<= 2.5e7).

@!multitest
domain I = read(1..=5_000)
let p: [I]1..=5_000 = read()
print(sum(min(I.before(i).count(j => p[j] > p[i]), I.after(i).count(j => p[j] > p[i])) for i in I))
