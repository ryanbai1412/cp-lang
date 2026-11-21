@! multitest
let n: Int = read()
var pos = n + 1
for i in 1..=n:
    if (n + 1) % i == 0 and (n + 1) % (i + 1) == 0:
        pos = i
        break
if pos == n + 1:
    print(-1)
else:
    let ans: List<Int> = [if j == pos: n elif j < pos: j else: j - 1 for j in 1..=n]
    print(ans)
