// expect: ok, output "40 20\n30"
// rule: 07.4.5
// A key taken from the view itself indexes it with no check (decision 388).
let a = [10, 20, 40, 30, 50]
let w = a[1..4]
let i = w.argmax()!
print(w[i], w.nth(0))
for (k, x) in w.items:
    if x == 30: print(w[k])
