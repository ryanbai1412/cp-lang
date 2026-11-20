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

constexpr int maxn = 3e5 + 5;

int n, m, arr[maxn][2];
bool val[maxn], vis[maxn];

bool nand(bool a, bool b) {
    return !(a & b);
}

bool dfs(int u) {
    if (vis[u] || u < n) {
        return val[u];
    }
    vis[u] = true;
    auto& [lc, rc] = arr[u];
    return val[u] = nand(dfs(lc), dfs(rc));
}

bool eval(const vector<bool>& v) {
    memset(vis, 0, sizeof(vis));
    copy(begin(v), end(v), val);
    return dfs(n + m - 1);
}

void solve() {
    cin >> n >> m;
    for (int i = 0; i < m; i++) {
        for (auto& a : arr[i + n]) {
            cin >> a;
            a--;
        }
    }
    bool b0 = eval(vector<bool>(n, false)), b1 = eval(vector<bool>(n, true));
    if (b0 == b1) {
        cout << string(n, '0') << endl;
        return;
    }
    auto check = [&](int x) -> bool {
        vector<bool> cur(n);
        fill(cur.begin(), cur.begin() + x, true);
        return eval(cur) == b0;
    };
    int l = 0, r = n - 1;
    while (l < r) {
        int mid = (l + r + 1) / 2;
        if (check(mid)) {
            l = mid;
        } else {
            r = mid - 1;
        }
    }
    string ans(n, '0');
    fill(ans.begin(), ans.begin() + l, '1');
    ans[l] = 'x';
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}