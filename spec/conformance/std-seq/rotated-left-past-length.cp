// expect: ok, output "YES YES YES"
let empty: List<Int> = []
print(yes([1, 2, 3].rotated_left(4) == [2, 3, 1]), yes([1, 2, 3].rotated_left(-1) == [3, 1, 2]), yes(empty.rotated_left(5) == []))
