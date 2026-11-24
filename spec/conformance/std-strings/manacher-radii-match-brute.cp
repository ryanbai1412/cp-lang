// expect: ok, output "YES"
// Every odd and even palindrome length and the total count agree with expanding by hand.
fn expand(s: Str, l0: Int, r0: Int) -> Int:
    var (l, r) = (l0, r0)
    while l >= 0 and r < s.len and s[l] == s[r]:
        l -= 1
        r += 1
    r - l - 1
var ok = true
for trial in 0..60:
    let n = rand(0, 40)
    let s = Str((0..n).map(_ => if rand(0, trial % 3 + 1) == 0: 'a' else: 'b'))
    let p = manacher(s)
    var total = 0
    for i in 0..n:
        if p.longest_at(i) != expand(s, i, i): ok = false
        total += (expand(s, i, i) + 1) / 2
        if i > 0:
            if p.longest_between(i) != expand(s, i - 1, i): ok = false
            total += expand(s, i - 1, i) / 2
    if p.count() != total: ok = false
print(if ok: "YES" else: "NO")
