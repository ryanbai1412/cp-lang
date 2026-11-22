// expect: ok, output "YES"
print(yes((1..4).map(x => x * x).collect() == [1, 4, 9]))
