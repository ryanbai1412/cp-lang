// expect: ok, output "3\n1 9\n9 25\na b"
// rule: 07.7.5
// `adjacent` is the std `Seq` body, so a range, a filtered list and a string pair up alike.
print((0..4).adjacent.count())
print([1, 4, 9, 16, 25].filter(x => x % 2 == 1).adjacent.collect())
print("ab".adjacent.collect())
