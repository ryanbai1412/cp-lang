// SAC 22 CC3 J4: convert a number from base b1 to base b2 (both at most 10).
for (b1, s, b2) in read [read Int](Int, Str, Int):
    print(s.parse_int(base: b1).to_str(base: b2))
