// expect: R0101
// rule: 04a.4.1
let a = [10, 20, 30, 40]
let blk = a[2..]
let i = 1
print(blk[blk.keys.first!])
print(blk[i]) // <- here
