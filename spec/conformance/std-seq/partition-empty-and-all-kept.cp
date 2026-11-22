// expect: ok, output "YES YES"
let empty: List<Int> = []
print(yes(empty.partition(x => x > 0) == ([], [])), yes([2, 4].partition(x => x % 2 == 0) == ([2, 4], [])))
