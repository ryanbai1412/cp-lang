// expect: ok, output "b 4 e bcde\ne 0\n0"
// rule: 04b.1.7 04b.5.9
// `rebase` is std's view for any Indexed source: a Str piece reads its keys from 0.
let t = "abcdef"[1..5].rebase
print(t[t.keys.first!], t.len, t.nth(3), Str(x for x in t))
let i: t.Idx = t.keys.first!
print(t[t.Idx.last!], Int(i))
let e: List<Int> = []
print(e.rebase.len)
