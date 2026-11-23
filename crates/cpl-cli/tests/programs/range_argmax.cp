// A range's keys are its elements (R04.4.31), so `argmax` on a range
// gives the element whose key is largest.
let a = [5, 9, 2]
print((0..a.len).argmax(by: i => a[i])!)
let r = 2..5
for (i, x) in r.items:
    print(i, x)
print((2..5).argmax(by: x => -x)!)
print((2..=6).argmin(by: x => (x - 4) * (x - 4))!)
print(yes((3..3).argmax(by: x => x) == None))
