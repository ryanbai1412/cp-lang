// Runnable port (proto/cp.py lib-run) of statement-first dmopc20c4p4.cp
// original: corpus/submissions/3485716.cpp
let n: Int = read()
var have = 0
var ans = 0
var spare: List<Int> = []
var out: List<Int> = []
for i in 0..n:
    let a: Int = read()
    while have < a:
        let j = spare[spare.len - 1]
        spare.pop()
        ans += i - j
        have += 1
    if a == have:
        have += 1
    else:
        ans += have - a - 1
        spare.push(i)
    out.push(ans)
for x in out: print(x)
