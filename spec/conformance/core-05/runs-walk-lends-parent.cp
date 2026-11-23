// expect: E0602 at 4
// rule: 05.7.4 05.9.1
// A run is a view of `a`; changing `a` inside the walk would change the runs being walked.
var a = [1, 1, 2]
for r in a.runs(by: x => x):
    a[2] = 1
    print(r.len)
