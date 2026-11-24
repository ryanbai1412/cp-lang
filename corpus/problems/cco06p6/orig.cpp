// CCC 2006 Stage 2 P6 Paint by Numbers: any grid matching the row and column clues. Each line is
// solved exactly given the cells fixed so far (a DP over cells and blocks says which cells can be
// a dot and which a star in some fill); lines are re-solved until nothing changes, then open cells are
// probed (see search), and only then does the solver guess a cell and backtrack.
#include <cstdio>
#include <algorithm>
#include <cstring>
#include <vector>

static int n, m;
static std::vector<int> rowc[32], colc[32];
using Grid = std::vector<int>;  // 0 open, 1 dot, 2 star

// cells: the line's states; writes back the forced ones. Returns false when the line has no fill.
static bool solveLine(int* cells, int len, const std::vector<int>& c, bool& changed) {
    int k = (int)c.size();
    // f[i][j]: the first i cells can hold the first j blocks, cell i-1 not a star of block j+1
    static bool f[34][34], g[34][34];
    static int dotRun[34];
    // dotRun[i]: cells i..i+len can be stars (no dot) — use prefix of "not dot"
    static int noDot[35];
    noDot[0] = 0;
    for (int i = 0; i < len; i++) noDot[i + 1] = noDot[i] + (cells[i] == 1);
    auto canStar = [&](int a, int b) { return noDot[b] - noDot[a] == 0; };  // [a, b)
    (void)dotRun;
    std::memset(f, 0, sizeof f);
    std::memset(g, 0, sizeof g);
    f[0][0] = true;
    for (int i = 0; i <= len; i++)
        for (int j = 0; j <= k; j++) {
            if (!f[i][j] || i == len) continue;
            if (cells[i] != 2) f[i + 1][j] = true;  // dot at i
            if (j < k) {
                int e = i + c[j];
                if (e <= len && canStar(i, e)) {
                    if (e == len) f[e][j + 1] = true;
                    else if (cells[e] != 2) f[e + 1][j + 1] = true;
                }
            }
        }
    if (!f[len][k]) return false;
    // g[i][j]: cells i.. can hold blocks j.. (suffix)
    g[len][k] = true;
    for (int i = len - 1; i >= 0; i--)
        for (int j = k; j >= 0; j--) {
            bool ok = cells[i] != 2 && g[i + 1][j];
            if (!ok && j < k) {
                int e = i + c[j];
                if (e <= len && canStar(i, e)) ok = e == len ? j + 1 == k : (cells[e] != 2 && g[e + 1][j + 1]);
            }
            g[i][j] = ok;
        }
    bool canD[34] = {}, canS[34] = {};
    static int starDiff[35];
    std::memset(starDiff, 0, sizeof starDiff);
    for (int i = 0; i < len; i++)
        for (int j = 0; j <= k; j++) {
            if (!f[i][j]) continue;
            if (cells[i] != 2 && g[i + 1][j]) canD[i] = true;
            if (j < k) {
                int e = i + c[j];
                if (e <= len && canStar(i, e)) {
                    bool ok = e == len ? j + 1 == k : (cells[e] != 2 && g[e + 1][j + 1]);
                    if (ok) {
                        starDiff[i]++, starDiff[e]--;
                        if (e < len) canD[e] = true;
                    }
                }
            }
        }
    int run = 0;
    for (int i = 0; i < len; i++) {
        run += starDiff[i];
        canS[i] = run > 0;
        if (!canD[i] && !canS[i]) return false;
        int want = canD[i] && canS[i] ? 0 : canD[i] ? 1 : 2;
        if (want && cells[i] != want) {
            if (cells[i]) return false;
            cells[i] = want, changed = true;
        }
    }
    return true;
}

// Re-solves the dirty lines until none is left; a cell a row fixes dirties its column and back.
static bool propagate(Grid& gr, unsigned rows, unsigned cols) {
    int line[32];
    while (rows || cols) {
        if (rows) {
            int i = __builtin_ctz(rows);
            rows &= rows - 1;
            for (int j = 0; j < m; j++) line[j] = gr[i * m + j];
            bool ch = false;
            if (!solveLine(line, m, rowc[i], ch)) return false;
            for (int j = 0; j < m; j++)
                if (line[j] != gr[i * m + j]) gr[i * m + j] = line[j], cols |= 1u << j;
        } else {
            int j = __builtin_ctz(cols);
            cols &= cols - 1;
            for (int i = 0; i < n; i++) line[i] = gr[i * m + j];
            bool ch = false;
            if (!solveLine(line, n, colc[j], ch)) return false;
            for (int i = 0; i < n; i++)
                if (line[i] != gr[i * m + j]) gr[i * m + j] = line[i], rows |= 1u << i;
        }
    }
    return true;
}

static unsigned allRows() { return n == 32 ? ~0u : (1u << n) - 1; }
static unsigned allCols() { return m == 32 ? ~0u : (1u << m) - 1; }

static bool propagate(Grid& gr) { return propagate(gr, allRows(), allCols()); }

// Fixes cell i to v and propagates from its row and column.
static bool assign(Grid& gr, int i, int v) {
    gr[i] = v;
    return propagate(gr, 1u << (i / m), 1u << (i % m));
}

static bool complete(const Grid& gr) {
    for (int v : gr)
        if (!v) return false;
    return true;
}

// Propagates, then probes each open cell both ways: a value that contradicts forces the other,
// and a cell both probes agree on is forced too. The guess is the cell whose weaker probe fixes
// the most cells.
static bool search(Grid gr, Grid& out) {
    if (!propagate(gr)) return false;
    int open = -1;
    for (bool changed = true; changed;) {
        changed = false;
        open = -1;
        int bestGain = -1;
        for (int i = 0; i < n * m; i++) {
            if (gr[i]) continue;
            Grid h[2] = {gr, gr};
            bool ok[2];
            int gain[2] = {0, 0};
            for (int t = 0; t < 2; t++) {
                ok[t] = assign(h[t], i, 2 - t);
                if (ok[t] && complete(h[t])) {
                    out = h[t];
                    return true;
                }
                if (ok[t])
                    for (int j = 0; j < n * m; j++) gain[t] += !gr[j];
                if (ok[t])
                    for (int j = 0; j < n * m; j++) gain[t] -= !h[t][j];
            }
            if (!ok[0] && !ok[1]) return false;
            if (!ok[0] || !ok[1]) {
                gr = h[ok[0] ? 0 : 1];
                changed = true;
                continue;
            }
            for (int j = 0; j < n * m; j++)
                if (!gr[j] && h[0][j] && h[0][j] == h[1][j]) gr[j] = h[0][j], changed = true;
            if (changed && !propagate(gr)) return false;
            int g = std::min(gain[0], gain[1]);
            if (!gr[i] && g > bestGain) bestGain = g, open = i;
        }
    }
    if (open < 0) {
        for (int i = 0; i < n * m && open < 0; i++)
            if (!gr[i]) open = i;
        if (open < 0) {
            out = gr;
            return true;
        }
    }
    for (int v : {2, 1}) {
        Grid h = gr;
        if (assign(h, open, v) && search(h, out)) return true;
    }
    return false;
}

int main() {
    std::scanf("%d %d", &n, &m);
    auto read = [](std::vector<int>& c) {
        for (int x; std::scanf("%d", &x) == 1 && x;) c.push_back(x);
    };
    for (int i = 0; i < n; i++) read(rowc[i]);
    for (int j = 0; j < m; j++) read(colc[j]);
    Grid out;
    if (!search(Grid(n * m, 0), out)) return 1;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) std::putchar(out[i * m + j] == 2 ? '*' : '.');
        std::putchar('\n');
    }
}
