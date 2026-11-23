import sys, random
# seed size: about `size` commands, inserts of ASCII 32-126 (spaces too) wrapped at random
# line breaks; size >= 1e5 gives 4000 edits, 2 MB inserted, many Moves, Prev/Next and Gets.
rng = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 100_000
ins_cap = 2 * 1024 * 1024 if big else rng.randint(1, 200)
edits_left = 4000 if big else rng.randint(1, max(1, S // 3))
moves_left, steps_left = (50_000, 200_000) if big else (S, S)
out_left = 3 * 1024 * 1024 if big else 10 ** 9
length, cur, inserted, cmds = 0, 0, 0, []
def text(n):
    return ''.join(chr(rng.choice([32] * 8 + list(range(32, 127)))) for _ in range(n))
total = 300_000 if big else S
while len(cmds) < total:
    r = rng.random()
    if edits_left and (r < 0.2 or length == 0) and inserted < ins_cap:
        edits_left -= 1
        if length == 0 or rng.random() < 0.7:
            per = (ins_cap - inserted) // max(1, edits_left + 1) if big else rng.randint(1, 30)
            n = max(1, min(ins_cap - inserted, per))
            t = text(n); inserted += n
            lines, i = [], 0
            while i < n:
                w = rng.randint(1, 80 if big else 8); lines.append(t[i:i + w]); i += w
            cmds.append(f"Insert {n}\n" + "\n".join(lines)); length += n
        elif cur < length:
            n = rng.randint(1, min(length - cur, 1000 if big else 10))
            cmds.append(f"Delete {n}"); length -= n
    elif r < 0.4 and moves_left:
        moves_left -= 1; cur = rng.randint(0, length); cmds.append(f"Move {cur}")
    elif r < 0.7 and steps_left:
        if cur > 0 and (cur == length or rng.random() < 0.5): cur -= 1; cmds.append("Prev")
        elif cur < length: cur += 1; cmds.append("Next")
        steps_left -= 1
    elif cur < length:
        n = rng.randint(1, min(length - cur, 40 if big else 10))
        if out_left < n + 1: break
        out_left -= n + 1; cmds.append(f"Get {n}")
    elif not (edits_left or moves_left or steps_left): break
print(len(cmds))
print("\n".join(cmds))
