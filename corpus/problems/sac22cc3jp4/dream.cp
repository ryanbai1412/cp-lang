// SAC 22 CC3 J4: convert a number from base b1 to base b2.
let q: 1..=1_000 = read()
for (b1, s, b2) in read<[q](b1: 2..=36, s: Str, b2: 2..=36)>():
    print(s.parse_int(base: b1)!.to_str(base: b2))
