// expect: ok, output "1 2 3 4 5\n1-2-3-4-5-6-7-8-9-10-11-12\n1 2 3 4 5 6"
// rule: R10.3.1
// `print`'s values come in through the `..xs` tuple, whatever their count, before its named parameters.
print(1, 2, 3, 4, 5)
print(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, sep: "-")
let xs = [1, 2, 3]
write(xs, 4, 5, 6)
print("")
