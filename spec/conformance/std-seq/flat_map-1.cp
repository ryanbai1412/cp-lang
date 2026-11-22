// expect: ok, output "YES"
print([1, 2].flat_map(x => 0..x).collect() == [0, 0, 1])
