// `Range.last_true` (spec/core/04-indices.md): `cpl check` accepts; `cpl run` has no such method.
print((1..=10).last_true(x => x * x <= 50)!)
