#!/usr/bin/env python3
"""fetch-statement.py ID...: save problem statements to corpus/statements/ID.md.

IDs follow the port names: cf2084a, agc079a. Codeforces pages are fetched
through a running Chrome (CDP on localhost:29229) because the site challenges
plain HTTP clients; run it as `uv run --with playwright python
scripts/fetch-statement.py ID...`. Statements are not ours to publish, so the
directory is git-ignored. `scripts/writable-from-docs.py` reads them.
"""
import asyncio, html, os, re, subprocess, sys
from html.parser import HTMLParser

OUT = os.path.join(os.path.dirname(__file__), "..", "corpus", "statements")


class Text(HTMLParser):
    def __init__(self):
        super().__init__()
        self.out = []

    def handle_starttag(self, tag, attrs):
        if tag in ("p", "div", "li", "br", "pre", "h1", "h2", "h3"):
            self.out.append("\n")
        if tag == "li":
            self.out.append("- ")
        if tag == "pre":
            self.out.append("```\n")

    def handle_endtag(self, tag):
        if tag == "pre":
            self.out.append("\n```\n")
        if tag in ("p", "div"):
            self.out.append("\n")

    def handle_data(self, data):
        self.out.append(data)


def text(fragment):
    p = Text()
    p.feed(fragment)
    t = "".join(p.out).replace("$$$$$$", "$$").replace("$$$", "$")
    return re.sub(r"\n{3,}", "\n\n", t).strip() + "\n"


def atcoder(pid):
    m = re.match(r"([a-z]+)(\d+)([a-z])$", pid)
    contest = m.group(1) + m.group(2)
    url = f"https://atcoder.jp/contests/{contest}/tasks/{contest}_{m.group(3)}"
    raw = subprocess.run(["curl", "-s", url], capture_output=True, text=True).stdout
    i = raw.find('<span class="lang-en">')
    if i < 0:
        return None
    title = html.unescape(re.search(r"<title>(.*?)</title>", raw).group(1))
    return f"# {pid} ({url})\n\n{title}\n\n" + text(raw[i:]).replace("\\(", "$").replace("\\)", "$")


async def codeforces(ids):
    from playwright.async_api import async_playwright

    async with async_playwright() as p:
        browser = await p.chromium.connect_over_cdp("http://localhost:29229")
        page = await browser.contexts[0].new_page()
        await page.goto("https://codeforces.com/", timeout=60000)
        await page.wait_for_timeout(3000)
        for pid in ids:
            m = re.match(r"cf(\d+)([a-z]\d?)$", pid)
            url = f"https://codeforces.com/contest/{m.group(1)}/problem/{m.group(2).upper()}"
            raw = await page.evaluate("u => fetch(u).then(r => r.text())", url)
            i = raw.find('class="problem-statement"')
            if i < 0:
                print("missing", pid)
            else:
                body = raw[raw.rfind("<div", 0, i) : raw.find("<script", i)]
                save(pid, f"# {pid} ({url})\n\n" + text(body))
            await page.wait_for_timeout(2500)
        await page.close()


def save(pid, s):
    os.makedirs(OUT, exist_ok=True)
    with open(os.path.join(OUT, pid + ".md"), "w") as f:
        f.write(s)
    print("saved", pid)


ids = [i for i in sys.argv[1:] if not os.path.exists(os.path.join(OUT, i + ".md"))]
for pid in [i for i in ids if not i.startswith("cf")]:
    s = atcoder(pid)
    save(pid, s) if s else print("missing", pid)
cf = [i for i in ids if i.startswith("cf")]
if cf:
    asyncio.run(codeforces(cf))
