// A guard after `,` in `if let` and `while let` is a whole condition, with
// `or`; `max` and `min` take any number of values.
let n: Int = read()
let a: List<Int> = [read() for _ in 0..n]
var stk: List<Int> = []
var i = 0
var pops = 0
while i <= n:
    while let top = stk.last, i == n or a[i] > a[top]:
        stk.pop()
        pops += 1
    stk.push(i)
    i += 1
let o: ?Int = if n > 2: Some(n) else: None
if let v = o, v == 9 or v == 5 and not (v < 0): print("five", v)
print(pops, max(1, 2, 3), min(4, 2, 3, 9), max(1.5, 2.5, 0.5), max(a[0], a[1], a[2], a[3]))
let some_n: ?Int = Some(n)
let no: ?Int = None
print(max(some_n, no, Some(9)), min(1, some_n, no), max(2, 7, 3, 7, 1), min(2.5, 1.0, 3.0))
if let v = o, v == 9 or v == 7: print("no")
elif let v = o, not (v == 9) and v == 5 or false: print("elif", v)
