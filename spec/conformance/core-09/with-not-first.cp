// expect: E0802
// rule: 09.1.1
print(1)
with io(sep: ",") // <- here
print(1, 2)
