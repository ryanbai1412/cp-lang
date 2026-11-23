let s: Str = read()
print("".join(["{r.first!}{r.len}" for r in s.runs()]))
