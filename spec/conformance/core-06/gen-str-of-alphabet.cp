// expect: ok
// rule: 06.4.3
// tool: stress --seed 1 --cases 50
// has: "cases agree"
@gen fn input():
    let s = gen<Str of "ab">(len: 1..=10)
    assert 1 <= s.len and s.len <= 10 and s.all(c => c == 'a' or c == 'b')
    print(s.len)
@alt(program)
fn brute():
    let k: Int = read()
    print(k)
let k: Int = read()
print(k)
