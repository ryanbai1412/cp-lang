// expect: ok, output "or skips\nand skips"
// rule: 01.5.7
let a = [1, 2]
let i = 2
if i >= a.len or a[i] > 0: print("or skips")
if not (i < a.len and a[i] > 0): print("and skips")
