import sys, random
# seed size: about `size` commands, inserts of ASCII 32-126 (spaces too) wrapped at random
# line breaks; size >= 1e5 gives 4000 edits, 2 MB inserted, many Moves, Prev/Next and Gets.
# --max: every bound at once: 4000 edits (3000 inserts of 2 MB in all at random cursors, so about
# 6000 pieces, and 1000 deletes), 50000 Moves, 200000 Prev/Next and Gets of up to 1500 characters
# until 3 MB is printed, all interleaved.
if sys.argv[1] == "--max":
    rng, cap, out_cap = random.Random(0), 2 * 1024 * 1024, 3 * 1024 * 1024
    ops = ["E"] * 3990 + ["M"] * 50_000 + ["S"] * 200_000 + ["G"] * 4000
    rng.shuffle(ops)
    ops = ["E"] * 10 + ops
    inserts, deletes, length, cur, inserted, printed, cmds = 3000, 1000, 0, 0, 0, 0, []
    pending = 0  # deletes waiting for text after the cursor
    for op in ops:
        if pending and cur < length:
            pending -= 1; deletes -= 1
            n = rng.randint(1, min(length - cur, 1000))
            cmds.append(f"Delete {n}"); length -= n
        if op == "E" and (inserts and (length < 2000 or not deletes or rng.random() < 0.75)):
            n = (cap - inserted) // inserts
            inserts -= 1; inserted += n
            t = "".join(chr(rng.randint(32, 126)) for _ in range(n))
            cmds.append(f"Insert {n}\n" + "\n".join(t[i:i + 80] for i in range(0, n, 80)))
            length += n
        elif op == "E":
            pending += 1
        elif op == "M":
            cur = rng.randint(0, length); cmds.append(f"Move {cur}")
        elif op == "S":
            if cur > 0 and (cur == length or rng.random() < 0.5): cur -= 1; cmds.append("Prev")
            else: cur += 1; cmds.append("Next")
        elif cur < length and printed < out_cap:
            n = min(length - cur, 1500, out_cap - printed - 1)
            if n > 0: printed += n + 1; cmds.append(f"Get {n}")
    print(len(cmds))
    print("\n".join(cmds))
    sys.exit()
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
