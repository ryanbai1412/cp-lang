// expect: ok, output "4"
// rule: 05.8.4
// A guard `k > l` proves `win[k]` and `win[l]` apart, so moving one slot into the other is no L0601.
var win = [[1], [2], [3], [4]]
let k = 3
let l = 1
if k > l:
    win[k] = win[l].take()
print(win.len)
