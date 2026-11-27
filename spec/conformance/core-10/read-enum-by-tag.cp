// expect: ok, output "W L W\nadd 5\nL none"
// rule: 10.2.1
// engines: next
// input: "WLW\n01 5\n"
// An enum reads one token and takes the variant whose tag it is: one-letter tags read one `Char`
// each, packed or spaced; an integer tag matches as a number, so `01` is `@tag(1)`; a token no
// tag names fails a read from a string. Interpreter and C++ agree.
enum R = @tag("W") Win | @tag("L") Lose
enum Q = @tag(1) Add(x: Int) | @tag(2) Ask
let rs = read<[3]R>()
print(rs[0], rs[1], rs[2])
match read<Q>():
    Add(x) => print("add", x)
    Ask => print("ask")
let bad = if let q = read<Q>(from: "3"): "some" else: "none"
print(read<R>(from: " L") ?? Win, bad)
