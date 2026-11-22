@!multitest
let n: 1..=6 = read()
let k: 1..=n = read()
var a: [0..n]1..=5 = read()

sort(mut a)
let mx = a[n - 1]
let c = count(x == mx for x in a)

if k < c:
    print("NO")
    return

print("YES")
var ans: List<Int> = []
for i in (n - k)..(n - c):
    ans.push(a[i])
ans.push(mx)
for i in (0..(n - k)).rev:
    ans.push(a[i])
for _ in 0..(c - 1):
    ans.push(mx)
print(ans)

@check
fn check():
    let verdict: Str = read_output()
    let top = max(a)
    let least = count(x == top for x in a)
    if verdict == "NO":
        assert k < least, "said NO, but {k} very pleasant donations are possible"
        return
    assert verdict == "YES", "the first line is YES or NO"
    let b: [0..n]Int = read_output()
    var got = [x for x in b]
    var want = [x for x in a]
    sort(mut got)
    sort(mut want)
    for i in 0..n: assert got[i] == want[i], "not a rearrangement of the donations"
    var pleasant = 0
    var total = 0
    for m in 0..n:
        if b[m] * m >= total: pleasant += 1
        total += b[m]
    assert pleasant == k, "{pleasant} very pleasant donations, not {k}"
