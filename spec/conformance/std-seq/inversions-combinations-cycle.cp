// expect: ok, output "YES 10 0 3\nYES\nYES NO NO\nYES 1 0\nYES\nYES"
use std.seq.{inversions, zip_exact, is_permutation}
print(inversions([3, 1, 2]) == 2, inversions([5, 4, 3, 2, 1]), inversions([1, 1, 1]), inversions([2, 1, 2, 1]))
print(zip_exact([1, 2], [3, 4]).collect() == [(1, 3), (2, 4)])
print(is_permutation([1, 2, 2], [2, 1, 2]) == true, is_permutation([1, 2], [1, 1]), is_permutation([1], [1, 1]))
print([1, 2, 3].combinations(2).collect() == [[1, 2], [1, 3], [2, 3]], [1, 2].combinations(0).collect().len, [1, 2].combinations(3).collect().len)
print((0..3).index_pairs().collect() == [(0, 1), (0, 2), (1, 2)])
domain V = 0..3
let p = [V] v => V((Int(v) + 1) % 3)
print([Int(v) for v in p.cycle(from: V(1))] == [1, 2, 0])
