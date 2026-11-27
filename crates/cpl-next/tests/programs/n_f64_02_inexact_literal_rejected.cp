// expect: E0302
// rule: R01.4.2
// An integer literal an `F64` cannot hold exactly is not silently rounded.
let r: F64 = 9007199254740993 // <- here
print(r)
