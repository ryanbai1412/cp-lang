// expect: ok, output "1 2 4 8"
// Acceptance for seq/U59 (infinite and repeating generators), handbook/product/std/sequences-usecases.md.
print(iterate(1, x => if x < 8: Some(x * 2) else: None).collect())
