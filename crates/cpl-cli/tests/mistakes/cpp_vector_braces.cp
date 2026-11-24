// here: 3
// C++'s `vector<int> v = {1, 2, 3};` (self-run mistakes): the set-literal fix gave `Set([1, 2, 3])`, which is no `List<Int>`.
vector<int> v = {1, 2, 3};
print(v.sum())
