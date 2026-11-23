// expect: ok, output "2-1-3\n2 1 3\n2;1"
// rule: 10.3.1
// `sep:` joins the elements of any sequence printed alone, a view such as `.rev` or a slice included; interpreter and C++ agree.
let xs = [3, 1, 2]
print(xs.rev, sep: "-")
print(xs.rev)
print(xs[1..3].rev, sep: ";")
