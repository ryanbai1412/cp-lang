// error: compile-time constant
// Synthesis #3 Leap 4: `if const` requires a compile-time constant condition.
let n: 1..=10 = read()
if const n > 0:
    print(1)
