// expect: ok, output "-1 99"
print((0..100).last_true(x => x < 0) ?? -1, (0..100).last_true(x => x >= 0) ?? -1)
