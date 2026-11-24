// An arrangement is a set of links between orthogonal neighbours of one team such that no
// tile has both a horizontal and a vertical link. Broken-profile DP over the narrower side w
// (w <= 17): bits 0..w-1 say the latest tile of each column links down, bit w says the left
// tile links right. The forward table f checks a link's colours at its later tile and the
// backward table g at its earlier tile, so neither f_k nor g_{k+1} looks at tile k's colour:
// A_k = sum over profiles s of f_k[s] * [links crossing the cut are one team] * (step k with
// tile k flipped, checking all its links) * g_{k+1}. g is kept at checkpoints and rebuilt one
// block at a time.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
const ll MOD = 1e9 + 7;
int n, m, W, S, cells;
vector<string> g;
char colour(int k, bool flip) {
    char c = g[k / W][k % W];
    return flip ? (c == 'N' ? 'S' : 'N') : c;
}
// Calls emit(next_state) for every move at tile k from state s.
enum { RECV = 1, SEND = 2 };
template <class F> void step(int k, int s, bool flip, int mode, F emit) {
    int i = k / W, j = k % W;
    bool up = s >> j & 1, left = s >> W & 1;
    if ((up && left) || (up && i == 0) || (left && j == 0)) return;
    char c = colour(k, flip);
    if ((mode & RECV) && up && colour(k - W, false) != c) return;
    if ((mode & RECV) && left && colour(k - 1, false) != c) return;
    bool sendR = j + 1 < W && !up && (!(mode & SEND) || colour(k + 1, false) == c);
    bool sendD = i + 1 < n && !left && (!(mode & SEND) || colour(k + W, false) == c);
    int base = s & ~(1 << j) & ~(1 << W);
    emit(base);
    if (sendR) emit(base | 1 << W);
    if (sendD) emit(base | 1 << j);
}
// Down links from the profile into tiles after k, other than tile k's own column.
bool crossing_ok(int k, int s) {
    int j = k % W;
    for (int c = 0; c < W; c++) {
        if (c == j || !(s >> c & 1)) continue;
        int from = c < j ? k - j + c : k - j + c - W;
        if (from + W >= cells || colour(from, false) != colour(from + W, false)) return false;
    }
    return true;
}
void back(int k, const vector<ll>& nxt, vector<ll>& cur) {
    for (int s = 0; s < S; s++) {
        ll v = 0;
        step(k, s, false, SEND, [&](int t) { v += nxt[t]; });
        cur[s] = v % MOD;
    }
}
int main() {
    int N, M;
    scanf("%d %d", &N, &M);
    vector<string> grid(N);
    for (auto& r : grid) {
        char buf[400];
        scanf("%s", buf);
        r = buf;
    }
    bool tr = M > N;
    if (tr) {
        n = M, W = N;
        g.assign(n, string(W, '?'));
        for (int i = 0; i < N; i++)
            for (int j = 0; j < M; j++) g[j][i] = grid[i][j];
    } else n = N, W = M, g = grid;
    cells = n * W, S = 1 << (W + 1);
    int B = max(1, (int)sqrt((double)cells));
    map<int, vector<ll>> ck;
    vector<ll> cur(S, 0), nxt(S, 0);
    nxt[0] = 1;
    ck[cells] = nxt;
    for (int k = cells - 1; k >= 0; k--) {
        back(k, nxt, cur);
        swap(cur, nxt);
        if (k % B == 0) ck[k] = nxt;
    }
    vector<ll> ans(cells), f(S, 0), f2(S);
    f[0] = 1;
    for (int b0 = 0; b0 < cells; b0 += B) {
        int b1 = min(cells, b0 + B);
        vector<vector<ll>> gs(b1 - b0 + 1);
        gs[b1 - b0] = ck[b1];
        for (int k = b1 - 1; k > b0; k--) {
            gs[k - b0].assign(S, 0);
            back(k, gs[k + 1 - b0], gs[k - b0]);
        }
        for (int k = b0; k < b1; k++) {
            const vector<ll>& gn = gs[k + 1 - b0];
            ll a = 0;
            fill(f2.begin(), f2.end(), 0);
            for (int s = 0; s < S; s++) {
                if (!f[s]) continue;
                ll fs = f[s], v = 0;
                if (crossing_ok(k, s)) {
                    step(k, s, true, RECV | SEND, [&](int t) { v += gn[t]; });
                    a = (a + fs * (v % MOD)) % MOD;
                }
                step(k, s, false, RECV, [&](int t) { f2[t] = (f2[t] + fs) % MOD; });
            }
            ans[k] = a;
            swap(f, f2);
        }
    }
    for (int i = 0; i < N; i++)
        for (int j = 0; j < M; j++) {
            int k = tr ? j * W + i : i * W + j;
            printf("%lld%c", ans[k], j + 1 < M ? ' ' : '\n');
        }
}
