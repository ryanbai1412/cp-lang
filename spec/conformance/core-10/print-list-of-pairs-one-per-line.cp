// expect: ok, output "1 2\n3 4\n1 2 3 4"
// rule: 10.3.5
// A list of pairs prints one pair per line, as pair input is laid out; `sep:` joins the pairs instead.
let ps = [(1, 2), (3, 4)]
print(ps)
print(ps, sep: " ")
