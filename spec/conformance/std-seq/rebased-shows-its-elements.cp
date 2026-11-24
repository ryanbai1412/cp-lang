// expect: ok, output "7 8 9\n[7 8 9]"
// rule: 08.1.2
// A `.rebase` view shows as its elements, space-separated, as a slice, `Rev` or `Stepped` view does.
let s = [5, 6, 7, 8, 9]
let t = s[2..].rebase
print(t)
print("[{t}]")
