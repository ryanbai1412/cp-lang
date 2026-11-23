// Codeforces 1932C: LR-remainders
// I/O: t tests; each n m, a_1..a_n, then a string s of n L/R commands; print, before each command, the product of the remaining array mod m.
@!multitest
let n: 1..=2e5 = read()
let m: 1..=1e4 = read()
let a = read<[n]1..=1e4>()
let s = read<Str>()
type Z = Mod(m)
// replay the removals, then multiply them back in reverse
var l = 0
var r = n
var gone = List<Int>()
for ch in s:
    if ch == 'L':
        gone.push(l)
        l += 1
    else:
        r -= 1
        gone.push(r)
var acc = Z.one
var out = List<Z>()
for i in gone.rev:
    acc *= Z(a[i])
    out.push(acc)
print(out.rev)
