// n: 2048
// `.items` as a value builds the list of (key, element) pairs. At n = 2048 that 32 KB block
// stays in cache at 4n too; next to the clones of builtin_array.cp at n = 16384 it moved
// Array.clone from 4x to 9x. Time it per call here: a whole process at small n is mostly
// startup, and read 50x at 4n where each call grows 3.5x. A quadratic reads 16x here and fails.
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let a: [n]Int = [i * 7919 % n for i in 0..n]
var t = ""
for i in 0..n: t.push(Char(97 + i * 7919 % 26))
var s = 0
for _ in 0..8192:
    s += xs.items.len // timed: builtin.List.items
    s += a.items.len // timed: builtin.Array.items
    s += t.items.len // timed: builtin.Str.items
print(s)
