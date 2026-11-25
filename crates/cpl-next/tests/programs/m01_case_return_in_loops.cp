// expect: ok, output "1 first 3\nnone 2\n3 first 7"
// input: "3\n3 1 3 5\n2 2 1\n1 7\n"
// A `return` in a test case ends that case from inside a loop over a range and a loop over a
// list; the state above the divider lives on.
var seen = 0
@!multitest
let n: Int = read()
var xs: List<Int> = List.new()
for i in 0..n:
    xs.push(read())
seen += 1
for x in xs:
    for j in 0..2:
        if x > 2 and j == 1:
            print(seen, "first", x)
            return
print("none", seen)
