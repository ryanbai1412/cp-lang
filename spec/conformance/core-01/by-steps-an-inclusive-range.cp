// expect: ok, output "2\n6\n10"
// rule: 01.8.4
// `by` steps an inclusive range too, ending on the last bound when it is hit.
for d in 2..=10 by 4:
    print(d)
