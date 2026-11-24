// expect: ok, output "4\n1\n3"
fn walk<S: Seq<Int>>(s: S):
    for x in s: print(x)
walk([3, 1, 4].rev)
