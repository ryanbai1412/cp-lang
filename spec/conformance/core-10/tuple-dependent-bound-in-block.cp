// expect: ok, output "2"
// input: "3 5\n"
// A tuple annotation inside a block may bound a part by an earlier one, as at top level (round 24, B2).
repeat 1:
    let (l, r): (1..=5, l..=5) = read()
    print(r - l)
