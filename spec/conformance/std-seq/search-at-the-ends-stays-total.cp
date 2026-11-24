// expect: ok, output "2 3 -1 -1 0 9"
// A monotone `ok` that holds everywhere or nowhere still answers at the ends; only the wrong way traps.
print([1, 3, 5, 7].bsearch(t => t >= 4).r, (0..10).last_true(x => x <= 3)!, (0..10).first_true(x => x > 20) ?? -1, (0..10).last_true(x => x > 20) ?? -1, (0..10).first_true(x => x >= 0)!, (0..10).last_true(x => x >= 0)!)
