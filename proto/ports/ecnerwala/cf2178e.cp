fn query(l: Int, r: Int) -> Int:
    print("?", l + 1, r)
    flush()
    read()

@!multitest
let n: Int = read()
var l = 0
var r = n
var tot = query(l, r)
while r - l > 1:
    var mi = l
    var ma = r - 1
    while ma - mi > 1:
        let md = (mi + ma) / 2
        let v = query(l, md)
        if 2 * v == tot:
            ma = md
            mi = md - 1
        elif 2 * v > tot:
            ma = md
        else:
            mi = md
    tot /= 2
    if r - ma <= ma - l: l = ma
    else: r = ma
print("!", tot)
flush()
