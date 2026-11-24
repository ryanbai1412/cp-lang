// An owned copy of a slice of n elements.
let n: Int = read()
let xs = [i * 7919 % n for i in 0..2 * n]
var s = 0
for r in 0..1024:
    s += xs[r % n..r % n + n].to_list().len // timed: seq.to_list
print(s)
