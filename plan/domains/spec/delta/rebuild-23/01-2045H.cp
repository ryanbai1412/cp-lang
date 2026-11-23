// CF 2045H Missing Separators: split S into the most strictly increasing words.
// dp[i, j] = most words for S[..j] whose last word is S[i..j]; the previous word
// S[k..i] fits iff it is smaller, which depends only on lcp(k, i) and the length b = j - i.
let s = read<Str>()
let n = s.len
var lcp: [0..=n, 0..=n]0..=5000 = 0
for k in 0..n:
    let z = z_function(s[k..])
    for i in (k + 1)..n:
        lcp[k, i] = z[i]
var dp: [0..=n, 0..=n]0..=5000 = 0
var par: [0..=n, 0..=n]0..=5000 = 0
for j in 1..=n:
    dp[0, j] = 1
for i in 1..n:
    // best[t]: best previous word that fits every current word longer than t
    var best: [0..=n]?(Int, Int) = None
    for k in 0..i if dp[k, i] > 0:
        let l = lcp[k, i]
        let t = if l >= i - k: i - k elif i + l < n and s[k + l] < s[i + l]: l else: n
        best[t] max= (Int(dp[k, i]), k)
    var run: ?(Int, Int) = None
    for b in 1..=(n - i):
        if let x = best[b - 1]:
            run max= x
        if let (c, k) = run:
            dp[i, i + b] = c + 1
            par[i, i + b] = k
let (cnt, start) = [(Int(dp[i, n]), i) for i in 0..n].max()!
var words: List<Str> = []
var i = start
var j = n
repeat cnt:
    words.push(Str(s[i..j]))
    let k = par[i, j]
    j = i
    i = k
print(cnt)
print(words.rev, sep: "\n")
