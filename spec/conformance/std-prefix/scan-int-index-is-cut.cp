// expect: ok, output "0 5 2 0\n0 5 2 0\n-3 0"
// rule: 07.1.6
// An `Int` index on a scan is the cut on a range-domain array and a `List` alike, through the last
// cut (decision 349); a named domain's position folds through itself.
let a: [0..3]Int = [5, -3, -2]
let pre = a.scan(+)
print([pre[i] for i in 0..=3])
let pb = [5, -3, -2].scan(+)
print([pb[i] for i in 0..=3])
domain I = 0..3
let b: [I]Int = [5, -3, -2]
let pc = b.scan(+)
print(pc[I(1)] - pc[I(1).l], pc[I(2)])
