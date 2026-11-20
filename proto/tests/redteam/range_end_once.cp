// A range is a value: `0..xs.len` reads its end once, so growing or shrinking `xs` in the body does not move it.
let xs = [1, 2]
var out = [0]
for x in xs:
    for i in 0..out.len:
        if out.len < 50: out.push(out[i] + x)
print(out.len)
var ys = [1, 2, 3, 4]
var t = 0
for i in 0..ys.len:
    if ys.len > 2: ys.pop()
    if i < ys.len: t += ys[i]
print(t)
