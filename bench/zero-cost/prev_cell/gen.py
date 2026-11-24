#!/usr/bin/env python3
import random
n = 5000
random.seed(2000)
alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
print(n)
print("".join(random.choice(alphabet) for _ in range(n)))
