// `wire.ask<Str>(q) == s` (the signature in std.io) parses as a chained comparison.
let wire = Wire.new()
print(wire.ask<Str>("? 1") == ">=")
