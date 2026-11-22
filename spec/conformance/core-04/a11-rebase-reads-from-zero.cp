// expect: ok, output "6 7"
// rule: 04a.11.5
let s = [5, 6, 7, 8]
let w = s[1..3]
print(w.rebase[0], w.rebase[1])
