// here: 4
// Python's `line.split()` (self-run mistakes): it was told `split` needs `sep`, with no fix.
let line = "3 4"
let a = [x.parse<Int>()! for x in line.split()]
print(a)
