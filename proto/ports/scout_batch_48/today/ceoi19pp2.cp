// CEOI '19 Practice P2 (Separator): online prefix-maximum set that drops every
// kept value larger than the newest element. Runnable on proto/cpc today.
// original: corpus/submissions/5343197.cpp
let q: 1..=1e6 = read()
var good = Set<Int>()
var mx = -1
var last = 0
for _ in 0..q:
    let raw: 0..=1e9 = read()
    let x = (raw + last) % 1e9
    if x > mx:
        mx = x
        good.insert(x)
    while good.len > 0 and good.max()! > x: good.remove(good.max()!)
    last = good.len
    print(last)
