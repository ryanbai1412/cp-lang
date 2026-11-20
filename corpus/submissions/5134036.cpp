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

constexpr int maxn = 2e4 + 305;

int n, m, arr[maxn][2];
bool val[maxn][2], vis[maxn];

bool nand(bool a, bool b) {
    return !(a & b);
}

void dfs1(int u) {
    if (vis[u] || u < n) {
        return;
    }
    vis[u] = true;
    if (u < n) {
        val[u][0] = false;
        val[u][1] = true;
        return;
    }
    auto& [lc, rc] = arr[u];
    dfs1(lc);
    dfs1(rc);
    for (int i = 0; i < 2; i++) {
        val[u][i] = nand(val[lc][i], val[rc][i]);
    }
}

void solve() {
    cin >> n >> m;
    for (int i = 0; i < m; i++) {
        for (auto& a : arr[i + n]) {
            cin >> a;
            a--;
        }
    }
    for (int i = 0; i < n; i++) {
        val[i][1] = true;
    }
    dfs1(n + m - 1);
    if (val[n + m - 1][0] == val[n + m - 1][1]) {
        cout << string(n, '0') << endl;
        return;
    }
    for (int b = 0; b < 2; b++) {
        memset(val, b, sizeof(val));
        for (int i = 0; i < n; i++) {
            val[i][0] = false;
            val[i][1] = true;
            memset(vis, 0, sizeof(vis));
            dfs1(n + m - 1);
            if (val[n + m - 1][0] == val[n + m - 1][1]) {
                string ans(n, char('0' + b));
                ans[i] = 'x';
                cout << ans << endl;
                return;
            }
            val[i][0] = val[i][1] = b;
        }
    }
    assert(false);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}