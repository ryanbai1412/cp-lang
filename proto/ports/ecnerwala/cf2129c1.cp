fn ask(inds: List<Int>) -> Int:
    print("?", inds.len, inds)
    flush()
    read()

@!multitest
let n: Int = read()
var ans: [0..n]Char = '?'
var mi = 0
var ma = n
while ma - mi > 1:
    let md = (mi + ma) / 2
    if ask([i + 1 for i in mi..=md]) > 0:
        ma = md
    else:
        mi = md
let open_i = mi + 1
let close_i = (mi + 1) % n + 1

fn check2(i1: Int, i2: Int):
    let r = ask([i1 + 1, close_i, open_i, i2 + 1, open_i, i2 + 1])
    ans[i1] = if r == 1 or r == 6: '(' else: ')'
    ans[i2] = if r == 0 or r == 1: '(' else: ')'

var i = 0
while i < n:
    check2(i, min(i + 1, n - 1))
    i += 2
print("!", Str(ans))
flush()
