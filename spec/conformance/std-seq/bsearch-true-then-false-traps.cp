// expect: R0401 (checked)
// A true-then-false `ok` (C++'s `lower_bound` habit, cf2159d2) traps instead of returning a wrong cut.
let st = [1, 3, 5, 7]
print(st.bsearch(t => t < 4).r) // <- here
