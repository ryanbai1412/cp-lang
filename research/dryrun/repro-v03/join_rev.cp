// `"".join(xs.rev)`: `cpl check` accepts; `cpl run` says `join` takes a list, "not a `List<Str>`".
let xs: List<Str> = ["a", "b"]
print("".join(xs.rev))
