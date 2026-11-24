// expect: E0503
// rule: 06.4.3
// tool: check
// has: "@alt(program)"
@brute(main) // <- here
fn brute():
    let n: 1..=8 = read()
    print(n)
let n: 1..=8 = read()
print(n)
