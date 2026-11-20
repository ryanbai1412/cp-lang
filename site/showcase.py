"""Render the CP Lang v2 Showcase from five verified corpus ports."""

import html
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
REPO_URL = "https://github.com/ryanbai1412/cp-lang/blob/main"

SHOWCASE_ITEMS = [
    dict(
        id="coci21c1p3",
        title="COCI '21 #1 Logičari",
        topic="Unicyclic tree DP · Dsu + Rooted + @memo",
        cp_path="proto/ports/scout_batch_35/coci21c1p3.cp",
        cpp_path="corpus/submissions/4080960.cpp",
        v2_loc=23,
        raw_loc=177,
        stripped_loc=115,
        summary="Drop one cycle edge with Dsu, root the remaining tree with Rooted, and let @memo solve the 4 end-color cases over ?Int without manual cycle extraction or sentinel 1e9 tables.",
        cpp_excerpt="""\
vector<int> cycle = scycle::compute();
for (auto& a : cycle) root[a] = true;
memset(memo, -1, sizeof(memo));
vector<array<int, 4>> cur;
for (auto& a : cycle)
    cur.push_back({dp(a, 0, 0), dp(a, 0, 1),
                   dp(a, 1, 0), dp(a, 1, 1)});
// ... plus 100 lines of cycle finder, tree DFS, and 4x4 ring DP
cout << solve(cur) << endl;""",
    ),
    dict(
        id="noip10p4",
        title="NOIP '10 Water Diversion",
        topic="Grid flood fill · (Row, Col) + p.adj4 + @memo",
        cp_path="proto/ports/scout_batch_37/noip10p4.cp",
        cpp_path="corpus/submissions/5205786.cpp",
        v2_loc=30,
        raw_loc=132,
        stripped_loc=104,
        summary="Product domain Cell = (R, C) and p.adj4 eliminate bounds helpers and 4-way boilerplate, while @memo computes each top cell's bottom-row interval directly.",
        cpp_excerpt="""\
bool ibs(int x, int y) {
    return 0 <= x && x < n && 0 <= y && y < m;
}
void dfs(int x, int y) {
    if (vis[x][y]) return;
    vis[x][y] = true;
    auto go = [&](int nx, int ny) {
        if (ibs(nx, ny) && arr[nx][ny] < arr[x][y])
            dfs(nx, ny);
    };
    go(x, y - 1); go(x, y + 1); go(x + 1, y); go(x - 1, y);
}
// ... plus bitset reachability table, pruning, and greedy cover""",
    ),
    dict(
        id="noi10p1",
        title="NOI '10 Energy Collection",
        topic="Harmonic GCD sieve · [1..=k]Int + .rev + .step(g)",
        cp_path="proto/ports/scout_batch_37/noi10p1.cp",
        cpp_path="corpus/submissions/5670455.cpp",
        v2_loc=9,
        raw_loc=66,
        stripped_loc=42,
        summary="A structural [1..=k]Int array, reversed inclusive range, and stepped generator comprehension replace a 40-line map sweep with nine lines of direct math.",
        cpp_excerpt="""\
long dp[n + 1] {};
for (int i = n; i >= 1; i--) {
    dp[i] = eval(n, m, i); // builds two vectors + std::map sweep
    for (int j = 2 * i; j <= n; j += i) {
        dp[i] -= dp[j];
    }
}
cout << dp[1] << endl;""",
    ),
    dict(
        id="coci21c6p2",
        title="COCI '21 #6 Zemljište",
        topic="2D sub-rectangle sweep · [0..n, 0..m] + nested fn",
        cp_path="proto/ports/scout_batch_35/coci21c6p2.cp",
        cpp_path="corpus/submissions/4529998.cpp",
        v2_loc=19,
        raw_loc=71,
        stripped_loc=41,
        summary="Structural 2D array typing, plain nested mutating fn offer, and parallel tuple assignment keep the two-pointer column sweep tight with zero sentinel 1e18.",
        cpp_excerpt="""\
long ans = 1e18;
auto upd = [&](long cx) { ans = min(ans, abs(cx - x) + abs(cx - y)); };
for (int l = 0; l < n; l++) {
    long cur[m] {};
    for (int r = l; r < n; r++) {
        for (int k = 0; k < m; k++) cur[k] += arr[r][k];
        int j = 0; long csum = 0;
        for (int i = 0; i < m; i++) {
            while (j <= i) csum += cur[j++];
            upd(csum);
            while (j < m && csum <= y) { csum += cur[j++]; upd(csum); }
            csum -= cur[i];
        }
    }
}""",
    ),
    dict(
        id="cheerio1s5",
        title="Cheerio '1 S5 Block Clearing",
        topic="Str + prefix + dual @memo interval & budget DP",
        cp_path="proto/ports/scout_batch_40/cheerio1s5.cp",
        cpp_path="corpus/submissions/5339756.cpp",
        v2_loc=16,
        raw_loc=74,
        stripped_loc=46,
        summary="Value-semantic 0..n-indexed Str, prefix(...), and two @memo recurrences replace nested reverse-loop DP tables and manual boundary guards.",
        cpp_excerpt="""\
int n = sz(s), psum[n + 1], dp[n][n] {};
for (int i = 0; i < n; i++) psum[i + 1] = psum[i] + (s[i] == 'H');
for (int l = n - 1; l >= 0; l--)
    for (int r = l; r < n; r++)
        for (int i = l; i <= r; i++) {
            if (s[i] != 'H') continue;
            int cans = r - l + 1;
            if (i > l) cans += dp[l][i - 1];
            if (i < r) cans += dp[i + 1][r];
            dp[l][r] = max(dp[l][r], cans);
        }
// ... plus second reverse DP table dp1[n + 1][kv + 1]""",
    ),
]


def _strip_header_comments(source):
    lines = source.splitlines()
    start = 0
    while start < len(lines) and (lines[start].startswith("//") or not lines[start].strip()):
        start += 1
    return "\n".join(lines[start:]).rstrip()


def showcase_html(highlight):
    total_v2 = sum(item["v2_loc"] for item in SHOWCASE_ITEMS)
    total_raw = sum(item["raw_loc"] for item in SHOWCASE_ITEMS)
    total_stripped = sum(item["stripped_loc"] for item in SHOWCASE_ITEMS)
    raw_pct = round((1 - total_v2 / total_raw) * 100)
    stripped_pct = round((1 - total_v2 / total_stripped) * 100)

    cards = []
    for item in SHOWCASE_ITEMS:
        cp_source = _strip_header_comments((REPO_ROOT / item["cp_path"]).read_text())
        raw_cut = round((1 - item["v2_loc"] / item["raw_loc"]) * 100)
        strip_cut = round((1 - item["v2_loc"] / item["stripped_loc"]) * 100)
        cards.append(
            f'<section class="showcase-card" id="showcase-{item["id"]}">'
            f'<div class="showcase-head">'
            f'<div><h3>{html.escape(item["title"])}</h3>'
            f'<p class="showcase-topic">{html.escape(item["topic"])}</p></div>'
            f'<div class="showcase-metrics">'
            f'<span class="metric-pill"><strong>{item["v2_loc"]}</strong> vs {item["raw_loc"]} raw (−{raw_cut}%)</span>'
            f'<span class="metric-pill"><strong>{item["v2_loc"]}</strong> vs {item["stripped_loc"]} stripped (−{strip_cut}%)</span>'
            f'</div></div>'
            f'<p class="showcase-summary">{html.escape(item["summary"])}</p>'
            f'<div class="showcase-grid">'
            f'<figure class="code-block"><figcaption><span>Before · C++17 ({item["raw_loc"]} raw / {item["stripped_loc"]} stripped) · <a href="{REPO_URL}/{item["cpp_path"]}">source ↗</a></span></figcaption>'
            f'<pre tabindex="0"><code>{html.escape(item["cpp_excerpt"])}</code></pre></figure>'
            f'<figure class="code-block"><figcaption><span>After · CP Lang v2 ({item["v2_loc"]} lines) · <a href="{REPO_URL}/{item["cp_path"]}">port ↗</a></span>'
            f'<button class="copy-button" type="button" aria-label="Copy code">Copy</button></figcaption>'
            f'<pre tabindex="0"><code>{highlight(cp_source, "cp")}</code></pre></figure>'
            f'</div></section>'
        )

    banner = (
        f'<div class="showcase-totals">'
        f'<span><strong>5 corpus ports:</strong> {total_v2} CP Lang v2 lines vs {total_raw} raw C++17 (−{raw_pct}%) and {total_stripped} library-stripped C++17 (−{stripped_pct}%).</span>'
        f'<span>Counts exclude blank and comment lines; every port has a verified runnable twin under <code>proto/ports/</code>.</span>'
        f'</div>'
    )
    return f'<div class="showcase">{banner}{"".join(cards)}</div>\n'
