// fixed: i
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
var s = 0
for r in 0..64:
    s += xs.lis().len // timed: seq.lis
    s += xs.spans().len // timed: seq.spans
    s += xs.prev_smaller().len // timed: seq.prev_smaller
    s += xs.next_smaller().len // timed: seq.next_smaller
    s += xs.perm_order() // timed: seq.perm_order
    s += xs.perm_sign() // timed: seq.perm_sign
print(s)
