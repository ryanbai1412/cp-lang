// expect: R0001
// rule: 01.5.8
// A negative power of an `Int` traps; it is not 0 and not a float.
let k = -1
print(2 ** k) // <- here
