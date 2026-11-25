// expect: ok, output "abc\nXY\nab"
// rule: R11.3.4
// `to_lower`, `to_upper` and `trim` are std.strings' methods; no `use` names them or their module.
print("AbC".to_lower())
print("xy".to_upper())
print("  ab ".trim())
