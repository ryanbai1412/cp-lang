// expect: L0601 at 5
// rule: 05.8.4
// With nothing known about `k` and `l` they may be the same slot, so the lint stays.
var win = [[1], [2], [3], [4]]
let k = read<Int>()
let l = read<Int>()
win[k] = win[l].take()
print(win.len)
