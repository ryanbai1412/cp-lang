// Runnable port (proto/cp.py lib-run) of mccc3j3.cp
// original: corpus/submissions/4223424.cpp

let (n, m): (Int, Int) = read()
let a: Str = read()
let b: Str = read()

domain Letter = 'A'..='Z'
var acnt: [Letter]Int = [Letter] _ => 0
var bcnt: [Letter]Int = [Letter] _ => 0
for i in 0..n: acnt[Letter(a[i])] += 1
for i in 0..m: bcnt[Letter(b[i])] += 1

var base = 0
var hi = 0
var lo: ?Int = None
for c in Letter:
    base += acnt[c] * bcnt[c]
    if acnt[c] > hi: hi = acnt[c]
    if bcnt[c] > 0:
        if let cur = lo:
            if acnt[c] < cur: lo = acnt[c]
        else:
            lo = acnt[c]

var gain = 0
if let x = lo:
    if hi - x > 0: gain = hi - x
print(base + gain)
