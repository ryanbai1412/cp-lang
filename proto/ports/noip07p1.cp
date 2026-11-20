// cp: 0.1 1f59ac0+dirty
// NOIP 2007 Counting Numbers: list each distinct number with how often it
// occurs, in increasing order.
let n: 1..=200000 = read()
let xs: [n]0..=1500000000 = read()
for same in runs(sorted(xs), same: (a, b) => a == b):
    print(same[0], same.len)
