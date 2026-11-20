// Codeforces 2268 D: from i you may step to any j < i or to the next greater
// element on the right; sum all finite pairwise distances.  The next-greater
// forest is cut into spines l -> .. -> r (r a prefix maximum); each spine edge
// i -> up[i] is charged for the pairs it separates, and the pairs strictly
// inside (i, up[i]] are split by distance 1, 2 or 3.
@!multitest
let n: 1..=5e5 = read()
let p: [0..n]Int = read()
var up: [0..n]Int = n       // next greater element to the right, n if none
var first: [0..n]Int = -1   // leftmost child in the next-greater forest
var kids: [0..n]Int = 0
var stk: List<Int> = []
for i in 0..n:
    while stk.len > 0 and p[stk.last!] < p[i]:
        let c = stk.pop()
        up[c] = i
        first[i] = c
        kids[i] += 1
    stk.push(i)

var ans = n * (n - 1) / 2   // every step to the left costs one
var l = 0
while l < n:
    var r = l
    while up[r] < n: r = up[r]
    var i = l
    while i != r:
        let u = up[i]
        let inside = u - i - 1
        ans += (i - l + 1) * (r - i)                  // pairs crossing this spine edge
        ans += (i - l + 1) * inside                   // then one final step down
        ans += (r - u) * (2 * inside - (kids[u] - 1)) // two steps up, sometimes one
        var two = sum(up[a] - a - 1 for a in i + 1..u)
        for b in i + 1..=u if first[b] >= 0:
            two += b - first[b] - kids[b]
            if b < u: two += kids[first[b]]
        let three = (u - i) * inside / 2 - inside - two
        ans += inside + 2 * two + 3 * three
        i = u
    l = r + 1
print(ans)
