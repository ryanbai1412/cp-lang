// expect: ok, output "0 3 3\n1 2 2\n2 1 1"
// rule: 05.6.2
// A view of a temporary keeps the temporary alive for as long as the view is used.
let a = [3, 1, 3, 2, 3, 2]
for (r, run) in a.sorted(by: desc).runs().items:
    print(r, run.first!, run.len)
