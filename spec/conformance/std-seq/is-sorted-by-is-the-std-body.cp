// expect: ok, output "YES NO YES\nYES NO\nYES YES"
print(yes([1, 2, 2].is_sorted()), yes([3, 1, 2].is_sorted()), yes(is_sorted([0, 5])))
print(yes([3, 2, 2].is_sorted(by: x => -x)), yes(["bb", "a"].is_sorted(by: s => s.len)))
print(yes(List<Int>().is_sorted()), yes((0..5).is_sorted()))
