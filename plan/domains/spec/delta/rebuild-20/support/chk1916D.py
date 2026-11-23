import sys, math
inp = open(sys.argv[1]).read().split(); out = open(sys.argv[2]).read().split()
t = int(inp[0]); at = 0
for c in range(1, t+1):
    n = int(inp[c]); nums = out[at:at+n]; at += n
    assert len(nums) == n, "short"
    assert len(set(nums)) == n, "dup"
    key = sorted(nums[0])
    for s in nums:
        assert len(s) == n and s[0] != '0' and sorted(s) == key, s
        r = math.isqrt(int(s)); assert r*r == int(s), s
assert at == len(out), "extra"
print("ok")
