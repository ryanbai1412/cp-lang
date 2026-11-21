@!multitest
let (n, k): (Int, Int) = read()
let a: [0..n]Int = read()
var freq: [0..2 * n]Int = 0
for x in a: freq[x - 1] += 1

var stk = [x for x in (2 * n..3 * n).rev]
var ans = 0
for v in (0..2 * n).rev:
    stk.push(v)
    while freq[v] > 0:
        if freq[v] == k: ans max= stk.last! - v
        stk.pop()
        freq[v] -= 1
print(ans)
