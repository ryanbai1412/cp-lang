@!multitest
let x: 1..=1000000000000 = read()
var cnt = 0
var d = 1
while d * d <= x:  // expect: trial_division
    if x % d == 0:
        cnt += if d * d == x: 1 else: 2
    d += 1
print(cnt)
