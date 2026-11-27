// expect: ok, output "5"
// rule: R11.3.4
// `hash_token` is std.strings' `Hashable` getter on `Int`. The program spells no name of
// std.strings, and the getter still resolves on its receiver's type.
print(5.hash_token)
