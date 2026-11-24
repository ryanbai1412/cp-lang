// `first` of a map stops at the first element only when the rest of the
// loop cannot trap: here the second element overflows, and both engines
// stop on it, as the interpreter's list of all of them does.
let xs: List<Int> = read()
print(xs.len)
print(xs.map(x => x * 4).first ?? -1)
