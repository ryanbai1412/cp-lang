// expect: ok, output "3\n1\n4"
fn back<S: Bidir<Int>>(s: S):
    for x in s.rev: print(x)
back([3, 1, 4].rev)
