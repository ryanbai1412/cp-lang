// expect: ok, output "YES"
print([1, 2, 5, 1].take_while(x => x < 3).collect() == [1, 2])
