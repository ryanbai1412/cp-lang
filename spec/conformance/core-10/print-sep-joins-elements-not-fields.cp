// expect: ok, output "1 2\n3 4"
// rule: 10.3.5
// `sep:` joins the elements of a sequence argument; the fields of each tuple are still joined by `io.sep`.
print([(1, 2), (3, 4)], sep: "\n")
