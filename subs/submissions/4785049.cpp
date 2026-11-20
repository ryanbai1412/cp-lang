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
#define sz(x) int((x).size())

const int dx[] = {0, 0, -1, 1}, dy[] = {-1, 1, 0, 0};

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

void solve() {
    int n, m, k;
    cin >> n >> m >> k;
    vector<vector<char>> arr(n, vector<char> (m));
    for (int i = 0; i < k; i++) {
        int x, y;
        cin >> x >> y;
        x--;
        y--;
        arr[x][y] = true;
    }
    vector<pair<int, int>> whites;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            if (!arr[i][j]) {
                whites.emplace_back(i, j);
            }
        }
    }
    auto ibs = [&](int x, int y) -> bool {
        return 0 <= x && x < n && 0 <= y && y < m;
    };
    int ans = 1e9;
    for (int i = 0; i < (1 << sz(whites)); i++) {
        auto toggle = [&]() -> void {
            for (int j = 0; j < sz(whites); j++) {
                if (on(i, j)) {
                    auto &[x, y] = whites[j];
                    arr[x][y] ^= true;
                }
            }
        };
        toggle();
        int white_cnt = 0, sx = -1, sy = -1;
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < m; j++) {
                if (!arr[i][j]) {
                    white_cnt++;
                    sx = i;
                    sy = j;
                }
            }
        }
        if (white_cnt <= 1) {
            continue;
        }
        int vis_cnt = 0;
        vector<pair<int, int>> q;
        vector<vector<char>> vis(n, vector<char> (m));
        auto push = [&](int x, int  y) -> void {
            if (!vis[x][y]) {
                vis_cnt++;
                vis[x][y] = true;
                q.emplace_back(x, y);
            }
        };
        push(sx, sy);
        while (sz(q)) {
            auto [x, y] = q.back();
            q.pop_back();
            for (int i = 0; i < 4; i++) {
                int cx = x + dx[i], cy = y + dy[i];
                if (ibs(cx, cy) && !arr[cx][cy]) {
                    push(cx, cy);
                }
            }
        }
        if (vis_cnt < white_cnt) {
            ans = min(ans, __builtin_popcount(i));
        }
        toggle();
    }
    if (ans == 1e9) {
        cout << -1 << endl;
    } else {
        cout << ans << endl;
    }
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}