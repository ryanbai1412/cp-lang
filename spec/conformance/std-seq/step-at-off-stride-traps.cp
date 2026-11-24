// expect: R0101 at 3
// rule: R04b.1.10
// A key the stride skips is not one of the view's keys, so reading it traps.
let a = [10, 20, 30, 40, 50]
print(a.step(2).at(3))
