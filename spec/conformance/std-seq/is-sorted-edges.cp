// expect: ok, output "YES YES NO YES"
let e: List<Int> = []
print(yes(e.is_sorted()), yes([7].is_sorted()), yes([2, 1].is_sorted()), yes([3, 3, 3].is_sorted()))
