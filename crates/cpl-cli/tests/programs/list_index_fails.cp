// A list index is checked in the C++ too: the last element of an empty
// list stops the program, it does not read before the buffer.
let n: Int = read()
var xs: List<Int> = []
for i in 0..n: xs.push(i * i)
var flags: List<Bool> = [true]
print(xs.len, flags[0])
xs[0] += 1
print(xs[xs.len - 1])
