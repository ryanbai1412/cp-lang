// here: 4
// no fix: `parse` of a value that is not a `Str` (self-run mistakes) said "`` is not in scope".
let x = 2.7
print(x.parse<Int>()!)
