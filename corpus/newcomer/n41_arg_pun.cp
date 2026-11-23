let (n, moves): (1..=100, 1..=100) = read()
fn walk(n: Int, moves: Int) -> Int = (n + moves) % 10
print(walk(n: n, moves: moves))  // expect: redundant_arg_name
