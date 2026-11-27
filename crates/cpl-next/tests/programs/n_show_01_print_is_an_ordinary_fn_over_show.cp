// expect: ok, output "1 a\n3 1 2 3\n1\n2\n-452\nn=7!\n3 x\n1 2\n3\nab\n-9223372036854775807 0 18446744073709551615\n1,2,3"
// rule: R10.3.1
// `print` packs its values into a tuple and writes it through `Show`; sequences join by a gap
// their elements choose, tuples by a space, and `sep:` replaces either.
let xs = [1, 2, 3]
print(1, "a")
print(xs.len, xs)
print([1, 2], sep: "\n")
print(-45, end: "")
print(2)
let n = 7
print("n={n}!")
print((3, "x"))
print([[1, 2], [3]])
print(['a', 'b'])
let m = -9223372036854775807
print(m, 0, U64(0) - U64(1))
print(1, 2, 3, sep: ",")
