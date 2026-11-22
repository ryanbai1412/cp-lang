#!/usr/bin/env python3
"""pick-ports.py SOURCE_REPO [N]: list unported Codeforces problems worth porting.

SOURCE_REPO is a clone of https://github.com/ecnerwala/contest-programming.
A candidate must have an accepted submission by the author on Codeforces
(the repo also holds unfinished drafts), a non-trivial original that prints
output, and no port yet in corpus/problems/. The Codeforces API is
read through a running Chrome (CDP on localhost:29229), as in
fetch-statement.py. Prints the newest N candidates, one per line.
"""
import asyncio, glob, json, os, re, sys

HANDLE = "ecnerwala"
PORTS = os.path.join(os.path.dirname(__file__), "..", "proto", "ports", "ecnerwala")


async def accepted():
    from playwright.async_api import async_playwright
    async with async_playwright() as p:
        browser = await p.chromium.connect_over_cdp("http://localhost:29229")
        page = await browser.contexts[0].new_page()
        await page.goto("https://codeforces.com/", wait_until="domcontentloaded")
        raw = await page.evaluate("u => fetch(u).then(r => r.text())",
                                  f"https://codeforces.com/api/user.status?handle={HANDLE}")
        await page.close()
    return {f"cf{s['problem'].get('contestId', 0)}{s['problem']['index']}".lower()
            for s in json.loads(raw)["result"] if s.get("verdict") == "OK"}


def main():
    repo, n = sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 24
    ok = asyncio.run(accepted())
    ported = {os.path.basename(f)[:-3] for f in glob.glob(os.path.join(PORTS, "*.cp"))}
    found = []
    for d in glob.glob(os.path.join(repo, "CF", "*", "*")):
        contest, letter = d.split(os.sep)[-2:]
        name = f"cf{contest}{letter}".lower()
        if not re.fullmatch(r"cf\d+[a-z]\d*", name) or name in ported or name not in ok:
            continue
        srcs = glob.glob(os.path.join(d, "*.cpp"))
        if not srcs:
            continue
        body = re.sub(r"//.*", "", open(srcs[0], errors="ignore").read())
        if len(body) < 1500 or not re.search(r"cout|printf|puts", body):
            continue
        found.append((int(contest), letter, name))
    for _, _, name in sorted(found, reverse=True)[:n]:
        print(name)


if __name__ == "__main__":
    main()
