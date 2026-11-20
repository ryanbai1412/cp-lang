#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int maxn = 16, dirs[4][2] {{0, 1}, {1, 0}, {-1, 0}, {0, -1}};

int dist[maxn][maxn][maxn][maxn];

void solve() {
    int n, sx, sy, tx, ty, e1x, e1y, e2x, e2y;
    cin >> n >> sx >> sy >> tx >> ty >> e1x >> e1y >> e2x >> e2y;
    sx--;
    sy--;
    tx--;
    ty--;
    e1x--;
    e1y--;
    e2x--;
    e2y--;
    auto ibs = [&](int x, int y) -> bool {
        return 0 <= x && x < n && 0 <= y && y < n;
    };
    queue<array<int, 4>> q;
    memset(dist, -1, sizeof(dist));
    auto push = [&](int x, int y, int ex, int ey, int d) -> void {
        int& cd = dist[x][y][ex][ey];
        if (cd != -1) {
            return;
        }
        cd = d;
        q.push({x, y, ex, ey});
    };
    push(sx, sy, e1x ^ e2x ^ sx, e1y ^ e2y ^ sy, 0);
    while (sz(q)) {
        auto [x, y, ex, ey] = q.front();
        q.pop();
        dbg(x, y, ex, ey);
        int d = dist[x][y][ex][ey];
        if (tie(x, y) == tie(tx, ty)) {
            cout << d << endl;
            return;
        }
        for (auto& [dx, dy] : dirs) {
            int cx = x + dx, cy = y + dy;
            if (!ibs(cx, cy)) {
                continue;
            }
            if (tie(cx, cy) == tie(ex, ey)) {
                push(ex, ey, x, y, d + 1);
            }
        }
        for (auto& [dx, dy] : dirs) {
            int cx = ex + dx, cy = ey + dy;
            if (!ibs(cx, cy)) {
                continue;
            }
            push(x, y, cx, cy, d + 1);
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}