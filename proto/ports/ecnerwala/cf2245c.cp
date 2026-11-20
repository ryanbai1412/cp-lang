// Codeforces 2245 C: build a permutation of 0..n whose suffix minima xor to
// k ^ n.  Besides 0 the suffix minima are the leftover value and maybe n - 1;
// everything else goes in front in decreasing order.
@!multitest
let (n, k): (1..=2e5, Int) = read()
let b = bit_width(n) - 1          // top bit of n
var want = k ^ n
var tail: List<Int> = []
if want >= 2 << b or (want >= 1 << b and n == 1 << b):
    print("NO")
    return
if want >= 1 << b:
    want ^= n - 1
    tail.push(n - 1)
if want > 0: tail.push(want)
tail.push(0)
var used: [0..n]Bool = false
for v in tail: used[v] = true
print("YES")
var perm = [i for i in (0..n).rev if not used[i]]
for v in tail.rev: perm.push(v)
print(perm)
