// `let reply: Str = wire.ask(q)` (spec/core/10-io.md): `cpl check` accepts; `cpl run` types `ask` as `Int`.
let wire = Wire.new()
let reply: Str = wire.ask("? 1")
print(reply)
