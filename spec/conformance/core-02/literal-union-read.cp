// expect: ok, output "del 2"
// input: "del\n"
// rule: 02.2.2 02.7.2
// Tokens that carry nothing read as a literal union, checked exhaustively by `match`.
let op = read<"add" | "del">()
let k = match op:
    "add" => 1
    "del" => 2
print(op, k)
