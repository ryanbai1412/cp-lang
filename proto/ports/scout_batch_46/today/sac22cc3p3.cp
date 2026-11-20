// SAC '22 CC3 P3 (today/ runnable subset) — original: corpus/submissions/4331598.cpp

let n: 1..=1000 = read()
let a: [0..n]Int = read()

var top = 0
for i in 0..n:
    top max= a[i]

// pass d orders by the last d digits; ties keep input order
var p = 10
var going = true
while going:
    var idx: List<Int> = []
    for i in 0..n:
        idx.push(i)
    sort(mut idx, by: i => (a[i] % p, i))
    var row: List<Int> = []
    for i in 0..n:
        row.push(a[idx[i]])
    print(row)
    going = top >= p
    p *= 10
