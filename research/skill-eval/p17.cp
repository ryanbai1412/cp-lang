let text = read<Str>()
let pat = read<Str>()
print(sum(1 for _ in text.find_all(pat)))
