let s: Str = read()
print(sum(r.len / 2 for r in s.runs() if r[0] == 'f'))
