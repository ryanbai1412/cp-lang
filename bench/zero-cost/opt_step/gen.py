#!/usr/bin/env python3
import random

random.seed(1)
print(1000, 1000)
print("".join(random.choice("<>^v") for _ in range(200000)))
