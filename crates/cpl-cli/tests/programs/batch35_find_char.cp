let s = "banana"
print(s.find('n') ?? -1, s.find("an") ?? -1, s.find('x') ?? -1, s.rfind('n') ?? -1)
print(s.find("") ?? -1, "".find('a') ?? -1, s.find("nab") ?? -1)
if let i = s.find('a'), let j = s.find("na"): print(i, j)
