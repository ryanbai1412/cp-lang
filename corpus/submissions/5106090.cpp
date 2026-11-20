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

#define int long

constexpr int maxn = 1005, dirs[4][2] = {{-1, 0}, {1, 0}, {0, -1}, {0, 1}};

void solve() {
    int n, kv;
    cin >> n >> kv;
    string arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int sx, sy, tx, ty;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            if (arr[i][j] == 'M') {
                arr[i][j] = 'G';
                sx = i;
                sy = j;
            } else if (arr[i][j] == 'D') {
                tx = i;
                ty = j;
            }
        }
    }
    auto ibs = [&](int x, int y) -> bool {
        return 0 <= x && x < n && 0 <= y && y < n;
    };
    int dist[maxn][maxn];
    memset(dist, 0x3f, sizeof(dist));
    {
        queue<pair<int, int>> q;
        auto push = [&](int x, int y, int d) -> void {
            if (d < dist[x][y]) {
                dist[x][y] = d;
                q.emplace(x, y);
            }
        };
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                if (arr[i][j] == 'H') {
                    push(i, j, 0);
                }
            }
        }
        while (sz(q)) {
            auto [x, y] = q.front();
            q.pop();
            for (auto& [dx, dy] : dirs) {
                int cx = x + dx, cy = y + dy;
                if (ibs(cx, cy) && arr[cx][cy] == 'G') {
                    push(cx, cy, dist[x][y] + 1);
                }
            }
        }
    }
    auto check = [&](int t) -> bool {
        bool vis[maxn][maxn] {};
        queue<array<int, 3>> q;
        auto push = [&](int x, int y, int d) -> void {
            if (!vis[x][y]) {
                vis[x][y] = true;
                q.push({x, y, d});
            }
        };
        push(sx, sy, kv * t);
        while (sz(q)) {
            auto [x, y, d] = q.front();
            if (x == tx && y == ty) {
                return true;
            }
            q.pop();
            for (auto& [dx, dy] : dirs) {
                int cx = x + dx, cy = y + dy;
                if (ibs(cx, cy) && (arr[cx][cy] == 'G' || arr[cx][cy] == 'D') &&
                    dist[cx][cy] > (d + 1) / kv) {
                    push(cx, cy, d + 1);
                }
            }
        }
        return false;
    };
    if (!check(1)) {
        cout << -1 << endl;
        return;
    }
    int l = 0, r = 1e9;
    while (l < r) {
        int mid = (l + r + 1) / 2;
        if (check(mid)) {
            l = mid;
        } else {
            r = mid - 1;
        }
    }
    cout << l << endl;
}

int32_t main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}