// expect: E0602 at 4
// rule: 05.7.4 05.9.1
// Writing `a` while a window of it is live is the same conflict as in `for x in a`.
var a = [3, 1, 2]
for w in a.windows(2):
    a[0] = 0
    print(w.first!)
