// expect: ok, output "4"
// rule: 05.8.4
// `for l in 0..k` knows `l < k`, so `win[k] = win[l].take()` reads and writes two different slots: no L0601.
var win = [[1], [2], [3], [4]]
for k in 1..4:
    for l in 0..k:
        win[k] = win[l].take()
print(win.len)
