// expect: ok, output "? 1\n6"
// rule: 10.4
// input: "5\n"
// The query is written before the read waits for its answer, and `flush()`
// changes no byte of what the judge gets.
print("? 1")
flush()
let x = read<Int>()
print(x + 1)
