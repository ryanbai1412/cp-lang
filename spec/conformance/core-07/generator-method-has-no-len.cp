// expect: E0606
// rule: 07.2.13
// `merge_sorted` is a generator called as a method: its `Gen<T>` is walk-only, so `.len` is the plain bound error, and its help names `.count()`.
let a = [1, 3, 5]
print(a.merge_sorted([2, 4]).len) // <- here
