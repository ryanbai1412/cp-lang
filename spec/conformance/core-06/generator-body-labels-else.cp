// expect: ok, output "2 2\ndone"
// rule: 06.3.3 07.6.1
fn upto(n: Int) -> yields Int:
    for i in 1..=n: yield i
'outer: for a in upto(3):
    for b in upto(3):
        if a * b == 4:
            print(a, b)
            break 'outer
for x in upto(2):
    if x > 5: break
else:
    print("done")
