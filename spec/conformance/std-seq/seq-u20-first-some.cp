// expect: ok, output "40"
// Acceptance for seq/U20 (the first `Some` a function gives), handbook/product/std/sequences-usecases.md.
print([1, 4, 9].first_some(x => if x > 3: Some(x * 10) else: None)!)
