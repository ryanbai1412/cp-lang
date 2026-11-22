// expect: ok, output "YES"
print([1, 2, 5, 1].drop_while(x => x < 3).collect() == [5, 1])
