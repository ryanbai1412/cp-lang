let n: Int = read()
var dp: [0..3, 0..n]Int = 0
var row: [0..n]Int = 0
for i in 0..n:
    row[i] = i * i
dp[1, ..] = row
let col: [0..3]Int = [7, 8, 9]
dp[.., 2] = col
let lst = [1, 2, 3, 4]
dp[0, ..] = lst
dp[2, 0..2] = [5, 6]
print(dp)
var a: [0..5]Int = [1, 2, 3, 4, 5]
a[1..3] = [7, 8]
a[3..] = lst[2..].copy()
print(a)
var b = [1, 2, 3, 4, 5]
b[..2] = [9, 9]
print(b)
var opt: [0..2, 0..3]?Int = None
let some_row: [0..3]?Int = [Some(1), None, Some(3)]
opt[1, ..] = some_row
print(opt[1, 0] ?? -1, opt[1, 1] ?? -1, opt[0, 2] ?? -1)
print("short")
dp[0, ..] = [1, 2]
print("not reached")
