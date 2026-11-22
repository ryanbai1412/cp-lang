// Baltic OI '08 P6 (Gloves): subset dominance boxes + Pareto skyline sweep.
// Runnable on proto/cpc today.
// original: corpus/submissions/5325518.cpp
let n: 1..=20 = read()
domain C = 0..n
let a: [C]Int = read()
let b: [C]Int = read()

var sa_tot = 0
var sb_tot = 0
for c in C:
    sa_tot += a[c]
    sb_tot += b[c]

var boxes: List<(Int, Int)> = [(0, sb_tot)]
for c in C:
    let cur = boxes.len
    for i in 0..cur:
        let p = boxes[i]
        boxes.push((p.0 + a[c], p.1 - b[c]))

boxes.sort()
let m = boxes.len
var suff: [0..m]Int = 0
for i in 0..m: suff[i] = boxes[i].1
for i in (0..m - 1).rev:
    suff[i] max= suff[i + 1]

var best_x = 0
var best_y = 0
var best_s: ?Int = none
for i in 0..m - 1:
    let x = boxes[i].0 + 1
    let y = suff[i + 1] + 1
    if x <= sa_tot and y <= sb_tot:
        if best_s == none or x + y < best_s!:
            best_s = x + y
            best_x = x
            best_y = y

print(best_x)
print(best_y)
