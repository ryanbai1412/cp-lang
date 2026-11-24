// expect: ok, output "YES 10 0 3\nYES\nYES NO NO\nYES 1 0\nYES\nYES"
use std.seq.{zip_exact, is_permutation}
print(yes(inversions([3, 1, 2]) == 2), inversions([5, 4, 3, 2, 1]), inversions([1, 1, 1]), inversions([2, 1, 2, 1]))
print(yes(zip_exact([1, 2], [3, 4]).collect() == [(1, 3), (2, 4)]))
print(yes([1, 2, 2].is_permutation([2, 1, 2]) == true), yes([1, 2].is_permutation([1, 1])), yes([1].is_permutation([1, 1])))
print(yes([1, 2, 3].combinations(2).collect() == [[1, 2], [1, 3], [2, 3]]), [1, 2].combinations(0).collect().len, [1, 2].combinations(3).collect().len)
print(yes((0..3).index_pairs().collect() == [(0, 1), (0, 2), (1, 2)]))
domain V = 0..3
let p = [V] v => V((Int(v) + 1) % 3)
print(yes([Int(v) for v in p.cycle(from: V(1))] == [1, 2, 0]))
