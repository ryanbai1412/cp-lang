let n: Int = read()
let xs = [i for i in 0..n]
let cs = Str(Char(97 + i % 26) for i in 0..n)
var s = 0
for i in 0..8 * n:
    let t = xs[i % n..].rebase // timed: seq.rebase
    s += t.nth(0)
    let u = cs[i % n..].rebase // timed: seq.rebase
    s += u.len
print(s)
