// cf1195e OpenStreetMap: sum of minima of every a x b subrectangle of a generated grid.
// Row pass: width-b sliding min per row; column pass: height-a sliding min over those.
let n: 1..=3000 = read()
let m: 1..=3000 = read()
let a: 1..=n = read()
let b: 1..=m = read()
var g: Int = read()
let x: Int = read()
let y: Int = read()
let z: 1..=1e9 = read()

domain R = 0..n
domain C = 0..m
domain C2 = 0..m - b + 1
var h: [R, C]0..1e9 = 0
for r in R:
    for c in C:
        h[r, c] = g
        g = (g * x + y) % z

var rowmin: [R, C2]0..1e9 = 0
for r in R:
    var w = Window(min)
    for c in C:
        w.push(h[r, c])
        if w.len > b: w.pop()
        if Int(c) + 1 >= b: rowmin[r, C2(Int(c) + 1 - b)] = w.all!

var total = 0
for c in C2:
    var w = Window(min)
    for r in R:
        w.push(rowmin[r, c])
        if w.len > a: w.pop()
        if Int(r) + 1 >= a: total += w.all!
print(total)
