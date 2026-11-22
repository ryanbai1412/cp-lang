// expect: E0503 at 1
// rule: 06.4.3
// tool: check
// has: "@alt(program)"
@brute(main)
fn brute():
    let n: 1..=8 = read()
    print(n)
let n: 1..=8 = read()
print(n)
