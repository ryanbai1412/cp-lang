// expect: ok, output "debc"
// rule: 04b.1.9 04.2.13
// A rebased Str piece is indexed by its own fresh keys from 0.
let s = "abcdef"
let t = s[1..5].rebase
print(Str(t[t.Idx.wrap(Int(i) + 2)] for i in t.keys))
