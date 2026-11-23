// Mock CCC '21 S1 (nccc8s1), runnable under proto/cp.py lib-run today
// (proto/cpc cannot index or slice unknown-length Str). See ../nccc8s1.cp.
// original: corpus/submissions/3381111.java
let al: Str = read()
let s: Str = read()
let n = s.len
var out: List<Str> = []
for i in 0..n:
    var del = ""
    for j in 0..n:
        if j != i: del.push(s[j])
    out.push(del)
    for k in 0..al.len:
        let c = al[k]
        var sub = ""
        for j in 0..n:
            if j == i: sub.push(c)
            else: sub.push(s[j])
        out.push(sub)
        var ins = ""
        for j in 0..n:
            if j == i: ins.push(c)
            ins.push(s[j])
        out.push(ins)
for k in 0..al.len:
    var app = ""
    for j in 0..n: app.push(s[j])
    app.push(al[k])
    out.push(app)
out.sort()
var prev: ?Str = None
for w in out:
    if w != s and prev != w:
        print(w)
        prev = w
