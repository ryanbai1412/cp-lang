"""End-to-end trace debugger checks in installed Google Chrome (see README)."""

import argparse
from contextlib import contextmanager
from copy import deepcopy
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import re
from shutil import which
import subprocess
import tempfile
from threading import Thread
from urllib.parse import urlsplit

from playwright.sync_api import expect, sync_playwright


ROOT = Path(__file__).resolve().parents[1]
SOURCE = """// Browser trace: deliberately disagree with the reference.
var score = 0
debug("begin", score)
score = 6
debug("score", score)
score += 1
debug("finish", score)
print(score)

@brute(main)
fn reference():
    var score = 0
    debug("begin", score)
    score = 3
    debug("score", score)
    score += 1
    debug("finish", score)
    print(score)
"""
DESKTOP = {"width": 1440, "height": 1240}
MOBILE = {"width": 390, "height": 844}


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_):
        pass


@contextmanager
def serve():
    handler = partial(QuietHandler, directory=ROOT / "site/_build")
    with ThreadingHTTPServer(("127.0.0.1", 0), handler) as server:
        worker = Thread(target=server.serve_forever, daemon=True)
        worker.start()
        try:
            yield f"http://127.0.0.1:{server.server_port}"
        finally:
            server.shutdown()
            worker.join()


@contextmanager
def checked_page(browser, theme, *, viewport=DESKTOP, origin=None):
    context = browser.new_context(color_scheme=theme, viewport=viewport)
    page = context.new_page()
    errors, forbidden = [], []
    page.on("pageerror", lambda error: errors.append(str(error)))

    def guard(route):
        url = urlsplit(route.request.url)
        local = f"{url.scheme}://{url.netloc}" == origin
        if url.scheme in {"http", "https"} and not local:
            forbidden.append(route.request.url)
            route.abort()
        else:
            route.continue_()

    context.route("**/*", guard)
    try:
        yield page
        assert not errors, errors
        assert not forbidden, forbidden
    finally:
        context.close()


def capture(page, path):
    if path:
        page.mouse.move(2, 2)
        expect(page.locator("#tip")).to_be_hidden()
        page.screenshot(path=path, full_page=True, animations="disabled")


def edit_and_trace(view):
    view.locator("[data-source]").fill(SOURCE)
    view.locator("[data-stdin]").fill("")
    view.locator("[data-trace]").click()


def assert_live_payload(payload):
    assert payload["source"]["code"] == SOURCE
    for lane, scores in [("fast", [0, 6, 7]), ("slow", [0, 3, 4])]:
        steps = {s.get("checkpoint"): s for s in payload["runs"][lane]["steps"]}
        for key, value in zip(("begin", "score", "finish"), scores):
            assert steps[f"debug {key}"]["values"][f"debug: {key}"] == value


def mismatch(view):
    expect(view.locator(".status-title")).to_have_text("First divergence at step 4")
    expect(view.locator("#step-number")).to_have_value("4")
    expect(view.locator("#diff-meta")).to_have_text("debug score · 1 differ")
    expect(view.locator("#selection")).to_contain_text("Fast 6")
    expect(view.locator("#selection")).to_contain_text("Reference 3")
    expect(view.locator("#source")).to_contain_text("deliberately disagree with the reference")


def jump(view, number):
    field = view.locator("#step-number")
    field.fill(str(number))
    field.press("Enter")
    expect(field).to_have_value(str(number))
    expect(view.locator("#scrub")).to_have_value(str(number - 1))


def scalar_geometry(view, label):
    results = view.locator("#lanes .scalar").evaluate_all("""nodes => nodes.map(card => {
        const box = card.getBoundingClientRect(), viz = card.closest('.viz');
        const clip = viz.getBoundingClientRect();
        const range = document.createRange();
        range.selectNodeContents(card);
        const glyph = range.getBoundingClientRect();
        return {
            value: card.textContent, card: box.height, client: viz.clientHeight,
            scroll: viz.scrollHeight, glyph: glyph.height, top: glyph.top - clip.top,
            bottom: clip.bottom - glyph.bottom
        };
    })""")
    assert [d["value"] for d in results] == ["6", "3"], results
    for d in results:
        assert d["card"] >= 48 and d["scroll"] == d["client"], results
        assert d["glyph"] > 0 and min(d["top"], d["bottom"]) >= 0, results
    print(label, "scalar geometry:", results)


def layout(page, label):
    page.evaluate("window.scrollTo(0, document.documentElement.scrollHeight)")
    dims = page.evaluate("""() => {
        const rect = s => document.querySelector(s).getBoundingClientRect();
        const shown = [...document.querySelectorAll('#view-state > :not([hidden])')].at(-1);
        const controls = ['#open', '#save-json', '#save-html', '#theme', '#step-number',
            '#first', '#last', '#checkpoint-prev', '#checkpoint', '#checkpoint-next'];
        const boxes = controls.map(s => [s, ...['left','top','right','bottom']
            .map(k => rect(s)[k])]);
        const statLines = [...document.querySelectorAll('#stats dt')]
            .map(dt => dt.offsetHeight / parseFloat(getComputedStyle(dt).lineHeight));
        return {width:innerWidth, height:innerHeight, scrollWidth:document.documentElement.scrollWidth,
            top:rect('.top').height, footer:rect('.timeline').height,
            clear:rect('.timeline').top - shown.getBoundingClientRect().bottom,
            boxes, statLines};
    }""")
    assert dims["scrollWidth"] == dims["width"] and dims["clear"] >= 0, dims
    assert all(0.9 <= lines <= 1.1 for lines in dims["statLines"]), dims
    for selector, left, top, right, bottom in dims["boxes"]:
        assert 0 <= left < right <= dims["width"], (selector, dims)
        assert 0 <= top < bottom <= dims["height"], (selector, dims)
    for i, first in enumerate(dims["boxes"]):
        for second in dims["boxes"][i + 1:]:
            assert (min(first[3], second[3]) <= max(first[1], second[1])
                    or min(first[4], second[4]) <= max(first[2], second[2])), (first, second)
    print(label, "layout:", {k: v for k, v in dims.items() if k != "boxes"})
    page.evaluate("window.scrollTo(0, 0)")


def site_live(browser, base, theme, folder, screenshots):
    with checked_page(browser, theme, origin=base) as page:
        page.goto(f"{base}/playground.html")
        edit_and_trace(page)
        expect(page).to_have_url(f"{base}/trace.html#live", timeout=15_000)
        expected = page.evaluate('JSON.parse(sessionStorage.getItem("cp-live-trace"))')
        assert_live_payload(expected)
        expect(page.locator("html")).to_have_attribute("data-theme", theme)
        mismatch(page)
        scalar_geometry(page, theme)
        layout(page, f"{theme} desktop live")
        capture(page, screenshots / f"live-wasm-{theme}.png" if screenshots else None)

        jump(page, 2)
        expect(page.locator("#diff-meta")).to_have_text("debug begin · all equal")
        expect(page.locator("#diffs")).to_contain_text("Every compared value matches")
        expect(page.locator("#checkpoint option:checked")).to_contain_text("= debug begin")
        jump(page, 5)
        expect(page.locator("#checkpoint")).to_have_value("")
        expect(page.locator("#diff-meta")).to_have_text("no checkpoint")
        expect(page.locator(".lane-note")).to_contain_text("checkpoint debug score")
        expect(page.locator('[data-lane="slow"] .scalar')).to_have_text("3")
        page.locator('[data-lane="fast"] .scalar').click()
        expect(page.locator("#selection")).to_contain_text("Held reference (not compared) 3")
        page.locator("#checkpoint-next").click()
        expect(page.locator("#step-number")).to_have_value("6")
        expect(page.locator("#diff-meta")).to_have_text("debug finish · 2 differ")
        page.locator("#checkpoint-prev").click()
        expect(page.locator("#step-number")).to_have_value("4")
        page.locator("#checkpoint").select_option("1")
        expect(page.locator("#step-number")).to_have_value("2")
        page.get_by_role("button", name="Speed 4×").click()
        page.locator("#play").click()
        expect(page.locator("#play")).to_have_attribute("aria-label", "Pause")
        expect(page.locator("#step-number")).to_have_value("8")
        expect(page.locator("#play")).to_have_attribute("aria-label", "Play")
        expect(page.locator("#tl-label")).to_contain_text("the program ended")

        page.locator("h1").click()
        page.keyboard.press("Home")
        expect(page.locator("#step-number")).to_have_value("1")
        page.keyboard.press("ArrowRight")
        expect(page.locator("#step-number")).to_have_value("2")
        page.keyboard.press("BracketRight")
        expect(page.locator("#step-number")).to_have_value("4")
        page.keyboard.press("BracketRight")
        expect(page.locator("#step-number")).to_have_value("6")
        page.keyboard.press("BracketLeft")
        mismatch(page)

        flipped = "dark" if theme == "light" else "light"
        page.locator("#theme").click()
        expect(page.locator("html")).to_have_attribute("data-theme", flipped)
        page.reload()
        expect(page.locator("html")).to_have_attribute("data-theme", flipped)
        mismatch(page)
        page.locator("#mode-single").click()
        expect(page.locator('[data-lane="slow"]')).to_have_count(0)
        expect(page.locator("#checkpoint")).to_have_value("3")
        page.locator("#mode-compare").click()
        mismatch(page)

        with page.expect_file_chooser() as chooser:
            page.locator("#open").click()
        chooser.value.set_files({"name": "invalid.json", "mimeType": "application/json",
                                 "buffer": b'{"schemaVersion":0}'})
        expect(page.locator("#toast")).to_contain_text("Could not open trace")
        mismatch(page)

        json_path = folder / f"capture-{theme}.json"
        html_path = folder / f"capture-{theme}.html"
        page.locator("#tab-log").click()
        expect(page.locator("#view-log")).to_be_visible()
        for selector, path in [("#save-json", json_path), ("#save-html", html_path)]:
            with page.expect_download() as exported:
                page.locator(selector).click()
            exported.value.save_as(path)
        assert json.loads(json_path.read_text()) == expected
        with page.expect_file_chooser() as chooser:
            page.locator("#open").click()
        chooser.value.set_files(str(json_path))
        expect(page.locator("#view-state")).to_be_visible()
        mismatch(page)

    with checked_page(browser, theme) as page:
        page.goto(html_path.as_uri())
        expect(page.locator("html")).to_have_attribute("data-context", "standalone")
        expect(page.locator("[data-site-nav]")).to_be_hidden()
        expect(page.locator("#view-state")).to_be_visible()
        expect(page.locator("#view-log")).to_be_hidden()
        assert json.loads(page.locator("#trace-data").text_content()) == expected
        mismatch(page)
    print(theme, "live WASM, playback, navigation, keyboard, import and exports passed")


def samples_and_mobile(browser, base, theme, screenshots):
    for viewport in [DESKTOP, MOBILE]:
        with checked_page(browser, theme, viewport=viewport, origin=base) as page:
            page.goto(f"{base}/trace.html")
            expect(page.locator(".status-title")).to_have_text("First divergence at step 5")
            expect(page.locator(".status-detail")).to_contain_text("dp[1][4]: fast 6 ≠ spec 3")
            expect(page.locator('#samples [aria-current="true"]')).to_contain_text("0/1 knapsack")
            expect(page.locator("#step-number")).to_have_value("5")
            expect(page.locator("#selection")).to_contain_text("Reference 3")
            expect(page.locator('[data-lane="fast"] .matrix .heat').first).to_be_visible()
            expect(page.locator('[data-lane="slow"] .matrix .expected')).to_have_text("3")
            name = "desktop" if viewport == DESKTOP else "mobile"
            layout(page, f"{theme} {name} knapsack")
            capture(page, screenshots / f"site-knapsack-after-{theme}-{name}.png" if screenshots else None)
            page.locator("#difference-focus").click()
            expect(page.locator("#difference-focus")).to_have_attribute("aria-pressed", "true")
            opacity = page.locator(".isolate .matrix .cell.zero").first.evaluate(
                "node => getComputedStyle(node).opacity")
            assert opacity == "0.25", opacity
            page.locator("#renderer").select_option("json")
            expect(page.locator("#difference-focus")).to_be_disabled()
            expect(page.locator('[data-lane="fast"] .matrix')).to_have_count(0)
            raw = page.locator('[data-lane="fast"] pre.json').text_content()
            assert json.loads(raw)[1][4] == 6
            page.locator("#renderer").select_option("auto")
            expect(page.locator('[data-lane="fast"] .matrix')).to_be_visible()
            if viewport == DESKTOP:
                for title, kind, widget in [
                    ("Shortest paths", "Graph", "svg.graph .node"),
                    ("Subtree sizes", "Rooted tree", "svg.graph .edge"),
                    ("Maze breadth-first search", "2D grid", ".matrix td"),
                    ("Insertion sort", "Indexed array", ".array .cell"),
                ]:
                    page.locator("#samples").get_by_role("button", name=re.compile(title)).click()
                    expect(page.locator("#var-type")).to_contain_text(kind)
                    expect(page.locator(f'[data-lane="fast"] {widget}').first).to_be_visible()
                graph_reference_highlights(page)
            else:
                page.goto(f"{base}/playground.html")
                edit_and_trace(page)
                expect(page).to_have_url(f"{base}/trace.html#live", timeout=15_000)
                mismatch(page)
                scalar_geometry(page, f"{theme} mobile")
                layout(page, f"{theme} mobile live")
    print(theme, "five automatic renderers, Raw JSON, isolate and responsive layout passed")


def graph_reference_highlights(page):
    # The site's graph example is single-run. Populate both lanes in a small
    # imported graph to check that the reference node and edge are marked.
    graph = json.loads((ROOT / "tools/trace/samples/dijkstra.json").read_text())
    fast = graph["runs"]["fast"]["steps"][0]
    fast["checkpoint"] = "graph"
    slow = deepcopy(fast)
    slow["values"]["graph"]["nodes"][0]["state"] = "visited"
    slow["values"]["graph"]["edges"][0]["weight"] += 1
    graph["runs"]["fast"]["steps"] = [fast]
    graph["runs"]["slow"] = {"label": "Reference graph", "steps": [slow]}
    with page.expect_file_chooser() as chooser:
        page.locator("#open").click()
    chooser.value.set_files({"name": "graph.json", "mimeType": "application/json",
                             "buffer": json.dumps(graph).encode()})
    expect(page.locator("#diff-meta")).to_have_text("graph · 2 differ")
    for widget in [".node", ".edge"]:
        expect(page.locator(f'[data-lane="fast"] {widget}.mismatch')).to_have_count(1)
        expect(page.locator(f'[data-lane="slow"] {widget}.expected')).to_have_count(1)


def offline_and_cli(browser, binary, folder):
    source = folder / "browser-regression.cp"
    source.write_text(SOURCE)
    stdin = folder / "empty.in"
    stdin.write_text("")
    cli_html = folder / "cli.html"
    cli_export = folder / "cli-reexport.html"
    with cli_html.open("wb") as output:
        subprocess.run([binary, "run", source, stdin, "--trace", "--format", "html"],
                       stdout=output, stderr=subprocess.PIPE, check=True)
    with checked_page(browser, "light") as page:
        page.goto(cli_html.as_uri())
        expect(page.locator("html")).to_have_attribute("data-context", "standalone")
        expect(page.locator("[data-site-nav]")).to_be_hidden()
        data = json.loads(page.locator("#trace-data").text_content())
        assert data["source"]["code"] == SOURCE and data["source"]["file"] == source.name
        assert set(data["runs"]) == {"fast"}
        expect(page.locator("#step-number")).to_have_attribute("max", "8")
        expect(page.locator("#checkpoint-nav")).to_be_hidden()
        jump(page, 7)
        expect(page.locator("#tl-label")).to_have_text("print → 7")
        # Unlike a divergent capture, this single run never calls the
        # first-divergence jump that would also switch the tab to State.
        page.locator("#tab-log").click()
        expect(page.locator("#view-log")).to_be_visible()
        with page.expect_download() as exported:
            page.locator("#save-html").click()
        exported.value.save_as(cli_export)
        page.goto(cli_export.as_uri())
        expect(page.locator("#tab-state")).to_have_attribute("aria-selected", "true")
        expect(page.locator("#view-state")).to_be_visible()
        expect(page.locator("#view-log")).to_be_hidden()
        assert json.loads(page.locator("#trace-data").text_content()) == data

        page.goto((ROOT / "tools/trace/index.html").as_uri())
        expect(page.locator(".status-title")).to_have_text("Ready for a capture")
        expect(page.locator(".timeline")).to_be_hidden()
        expect(page.locator("#save-html")).to_be_disabled()
        with page.expect_file_chooser() as chooser:
            page.locator("#open").click()
        chooser.value.set_files(str(cli_html))
        expect(page.locator("#title")).to_have_text(source.name)
        expect(page.locator("#view-state")).to_be_visible()
        expect(page.locator(".timeline")).to_be_visible()
    print("fresh CLI capture and recovery from empty standalone template passed")

    # The common fixture expects no input. Pass a generator that prints nothing.
    generator = which("true")
    assert generator, "an executable `true` is needed as the empty stress input generator"
    stress = subprocess.run(
        [binary, "stress", source, "--gen", generator, "--cases", "1", "--seed", "1",
         "--jobs", "1", "--trace"], capture_output=True, text=True, timeout=30)
    stress_html = source.with_suffix(".tests") / "stress-1.trace.html"
    assert stress_html.is_file(), stress.stdout + stress.stderr
    with checked_page(browser, "dark") as page:
        page.goto(stress_html.as_uri())
        assert_live_payload(json.loads(page.locator("#trace-data").text_content()))
        mismatch(page)
        jump(page, 2)
        expect(page.locator("#diff-meta")).to_have_text("debug begin · all equal")
    print("fresh CLI stress capture with agreeing and divergent checkpoints passed")

    for theme in ["light", "dark"]:
        with checked_page(browser, theme) as page:
            page.goto((ROOT / "dist/cp-lang.html").as_uri())
            frame = page.frame_locator("#workspace")
            frame.get_by_role("link", name="Playground", exact=True).first.click()
            edit_and_trace(frame)
            expect(page).to_have_url(re.compile(r"#trace\.html#live$"), timeout=15_000)
            assert_live_payload(page.evaluate("JSON.parse(window.cpLiveTrace)"))
            mismatch(frame)
            expect(frame.locator("html")).to_have_attribute("data-theme", theme)
        print(theme, "offline file:// WASM and parent.cpLiveTrace passed")


CALL = """fn evens() -> Int:
    var picked = [i for i in 1..=4 if i % 2 == 0]
    var count = 0
    for i in picked:
        count += 1
    return count

print(evens())
""" + "".join(f"// padding {i}\n" for i in range(60))


def call_locals_and_scroll(browser, binary, folder):
    source = folder / "call.cp"
    source.write_text(CALL)
    html = folder / "call.html"
    with html.open("wb") as output:
        subprocess.run([binary, "run", source, "--trace", "--format", "html"],
                       stdin=subprocess.DEVNULL, stdout=output, stderr=subprocess.PIPE, check=True)
    with checked_page(browser, "light", viewport={"width": 1024, "height": 700}) as page:
        page.goto(html.as_uri())
        data = json.loads(page.locator("#trace-data").text_content())
        steps = data["runs"]["fast"]["steps"]
        jump(page, len(steps))
        expect(page.locator("#tl-label")).to_have_text("the program ended")
        counted = next(i for i, s in enumerate(steps) if s["line"] == 3) + 1
        jump(page, counted)
        expect(page.locator("#locals")).to_contain_text("[2, 4]")
        expect(page.locator("#frames")).to_contain_text("evens")
        visible = page.evaluate("""() => {
            const box = document.getElementById('source'), on = box.querySelector('.line.on');
            const a = box.getBoundingClientRect(), b = on.getBoundingClientRect();
            return b.top >= a.top && b.bottom <= a.bottom;
        }""")
        assert visible, "the current source line is scrolled out of view"
    print("call locals show lists as lists and the source follows the current line")


LIVE = """let n: 1..=10 = read()
var total = 0
for i in 1..=n:
    total += i
print(total)
"""


def live_rerun(browser, binary, folder):
    source = folder / "live.cp"
    source.write_text(LIVE)
    (folder / "live.tests").mkdir()
    (folder / "live.tests/stress-1.in").write_text("4\n")
    server = subprocess.Popen([binary, "run", source, "--trace", "--live"],
                              stderr=subprocess.PIPE, text=True)
    try:
        url = re.search(r"http://\S+", server.stderr.readline()).group(0)
        assert "stress-1.in" in server.stderr.readline()
        with checked_page(browser, "light", origin=url.rstrip("/")) as page:
            page.goto(url)
            data = json.loads(page.locator("#trace-data").text_content())
            lines = [s["line"] for s in data["runs"]["fast"]["steps"]]
            third = [i for i, line in enumerate(lines) if line == 4][2] + 1
            jump(page, third)
            source.write_text(LIVE.replace("total += i", "total += 2 * i"))
            expect(page.locator("#toast")).to_have_text("Re-ran after your edit")
            expect(page.locator(".status-title")).to_contain_text("first change at step")
            expect(page.locator("#step-number")).to_have_value(str(third))
            expect(page.locator("#source")).to_contain_text("total += 2 * i")
            page.locator("#go-change").click()
            changed = [i for i, line in enumerate(lines) if line == 4][0] + 1
            expect(page.locator("#step-number")).to_have_value(str(changed))
            source.write_text(LIVE + "fn (\n")
            expect(page.locator("#toast")).to_contain_text("kept the last good run")
            expect(page.locator("#source")).to_contain_text("total += 2 * i")
    finally:
        server.kill()
        server.wait()
    print("live re-run on the pinned input keeps the place and shows the first change")


REPLAY = """let n: 1..=10 = read()
var total = 0
fn add(i: Int) -> Int:
    total += i
    return total
for i in 1..=n:
    print(add(i))
"""


def replay_second_call(page, data):
    steps = data["runs"]["fast"]["steps"]
    second = next(i for i, s in enumerate(steps)
                  if (s.get("frames") or [{}])[0].get("fn") == "add" and s["values"].get("total") == 3)
    jump(page, second + 1)
    page.get_by_role("button", name="Replay this call").click()
    expect(page.locator("#replay-title")).to_have_text("Replay this call of add")
    box = page.locator("#replay-source")
    box.fill(REPLAY.replace("return total", "return total * 10"))
    box.press("Control+Enter")
    report = page.locator("#replay-report")
    expect(report).to_contain_text("replayed call 2 of `add(i: 2)`")
    expect(report).to_contain_text("returns 30 (before the edit: 3)")
    expect(report).to_contain_text("first change: step")
    clear = page.evaluate("""() => document.querySelector('.timeline').getBoundingClientRect().top
        - document.getElementById('replay-report').getBoundingClientRect().bottom""")
    assert clear >= 0, f"the replay report is {-clear}px under the timeline"
    box.fill("fn (\n")
    page.locator("#replay-run").click()
    expect(report.locator(".bad")).to_contain_text("edited file does not build")


def live_replay(browser, binary, folder):
    source = folder / "replay.cp"
    source.write_text(REPLAY)
    (folder / "replay.tests").mkdir()
    (folder / "replay.tests/stress-1.in").write_text("3\n")
    server = subprocess.Popen([binary, "run", source, "--trace", "--live"],
                              stderr=subprocess.PIPE, text=True)
    try:
        url = re.search(r"http://\S+", server.stderr.readline()).group(0)
        with checked_page(browser, "light", origin=url.rstrip("/")) as page:
            page.goto(url)
            replay_second_call(page, json.loads(page.locator("#trace-data").text_content()))
            assert source.read_text() == REPLAY, "replaying never touches the file"
            at = page.locator("#step-number").input_value()
            source.write_text(REPLAY.replace("add", "plus"))
            expect(page.locator("#source")).to_contain_text("fn plus", timeout=10_000)
            expect(page.locator("#replay-panel")).to_be_hidden()
            expect(page.locator("#step-number")).to_have_value(at)
    finally:
        server.kill()
        server.wait()
    print("replay this call reruns one invocation with the edited fn and reports the change")


def wasm_replay(browser, base):
    with checked_page(browser, "dark", origin=base) as page:
        page.goto(f"{base}/playground.html")
        page.locator("[data-source]").fill(REPLAY)
        page.locator("[data-stdin]").fill("3\n")
        page.locator("[data-trace]").click()
        expect(page).to_have_url(f"{base}/trace.html#live", timeout=15_000)
        expect(page.locator("#source")).to_contain_text("fn add")
        replay_second_call(page, page.evaluate('JSON.parse(sessionStorage.getItem("cp-live-trace"))'))
    print("replay this call in the browser runs the same engine through WebAssembly")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--chrome", required=True, help="path to installed Google Chrome")
    parser.add_argument("--cpl", type=Path, default=ROOT / "target/debug/cpl")
    parser.add_argument("--screenshots", type=Path, help="optional screenshot output directory")
    args = parser.parse_args()
    if args.screenshots:
        args.screenshots.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp, serve() as base, sync_playwright() as p:
        browser = p.chromium.launch(executable_path=args.chrome, headless=True)
        print("Headless Chrome", browser.version)
        for theme in ["light", "dark"]:
            site_live(browser, base, theme, Path(tmp), args.screenshots)
            samples_and_mobile(browser, base, theme, args.screenshots)
        offline_and_cli(browser, args.cpl.resolve(), Path(tmp))
        call_locals_and_scroll(browser, args.cpl.resolve(), Path(tmp))
        live_rerun(browser, args.cpl.resolve(), Path(tmp))
        live_replay(browser, args.cpl.resolve(), Path(tmp))
        wasm_replay(browser, base)
        browser.close()


if __name__ == "__main__":
    main()
