// expect: ok, output "0\n1\n2\ndone"
// rule: 01.7.4
fn upto(n: Int) -> yields Int:
    for i in 0..n: yield i
for x in upto(10):
    if x == 3: break
    print(x)
else:
    print("never")
for x in upto(2):
    if x == 5: break
else:
    print("done")
