// The sum of 3, 5, ... below 1e12 is about 2.5e23: it traps, as the loop would.
let m: 1..=1e18 = read()
print((3..m).step(2).sum())
