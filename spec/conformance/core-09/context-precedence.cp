// expect: ok, output "1,2\n1-2\n1;2\n1,2"
// rule: 09.1.1
with io(sep: ",")
print(1, 2)
print(1, 2, sep: "-")
with io(sep: ";"):
    print(1, 2)
print(1, 2)
