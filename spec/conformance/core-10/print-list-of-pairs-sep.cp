// expect: ok, output "1 2\n3 4"
// rule: 10.3.5
// `sep:` separates the elements; each pair's fields still use `io.sep` (rebuild 7, H-PRINT-TUPLE-LIST).
let ps = [(1, 2), (3, 4)]
print(ps, sep: "\n")
