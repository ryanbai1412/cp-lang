// Codeforces 2092D Mishkin Energizer: https://codeforces.com/problemset/problem/2092/D
// Impossible iff every character is the same. Otherwise raise the rarest letter x: insert it between
// two different letters that are not x; if every unequal pair touches x, first insert the third
// letter into such a pair, which creates a pair without x.
@!multitest
let n: 1..=100 = read()
let s = read<Str(len: n..=n) of "LIT">()
if s.all(c => c == s[0]):
    print(-1)
    return
var t = List(s)
var ops = List<Int>()
fn third(a: Char, b: Char) -> Char = "LIT".find(c => c != a and c != b)!
fn insert_at(i: Int):
    // between t[i - 1] and t[i]; the judge numbers this gap i (1-based)
    t.insert(i, third(t[i - 1], t[i]))
    ops.push(i)
while not (t.count('L') == t.count('I') and t.count('I') == t.count('T')):
    let x = "LIT".min(by: c => t.count(c))!
    if let i = (1..t.len).find(i => t[i - 1] != t[i] and t[i - 1] != x and t[i] != x):
        insert_at(i)
    else:
        insert_at((1..t.len).find(i => t[i - 1] != t[i])!)
print(ops.len)
for i in ops: print(i)

@check
fn ok():
    let n: Int = read()
    let s: Str = read()
    let m: Int = read_output()
    if m == -1:
        assert s.all(c => c == s[0]), "said impossible, but {s} has two letters"
        return
    assert 0 <= m <= 2 * n, "{m} operations"
    var t = List(s)
    repeat m:
        let i: Int = read_output()
        assert 1 <= i < t.len, "gap {i} outside 1..{t.len - 1}"
        assert t[i - 1] != t[i], "gap {i} sits between equal letters"
        t.insert(i, "LIT".find(c => c != t[i - 1] and c != t[i])!)
    assert t.count('L') == t.count('I') and t.count('I') == t.count('T'), "not balanced"
