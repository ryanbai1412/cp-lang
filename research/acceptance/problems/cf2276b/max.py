import random
random.seed(1)
tests = ["0" * 60000, "".join("01"[i % 2] for i in range(60000)), "".join(random.choice("01") for _ in range(60000)), "1" + "0" * 59998 + "1"]
tests += [random.choice(["000", "010", "001", "111", "101"]) for _ in range(9996)]
assert len(tests) <= 10**4 and sum(map(len, tests)) <= 3 * 10**5
print(len(tests)); print("\n".join(tests))
