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

constexpr int maxn = 5e5 + 5, maxd = 21;
//constexpr int maxn = 5e3 + 5, maxd = 21;

bool arrb[maxn];
int kv, arr[maxn], memo[maxn][2 * maxd + 5];
vector<int> graph[maxn];

void pdfs(int u) {
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        pdfs(v);
    }
}

int dp(int u, int d) {
    int& ans = memo[u][d + maxd];
    if (ans != -1) {
        return ans;
    }
    ans = 1e9;
    if (d < kv) {
        ans = min(ans, dp(u, d + 1));
    }
    if (d == kv) {
        int cans = arr[u];
        for (auto& v : graph[u]) {
            cans += dp(v, -kv);
        }
        ans = min(ans, cans);
    } else if (d >= 0) {
        if (sz(graph[u])) {
            int csum = 0, mdiff = 1e9;
            for (auto& v : graph[u]) {
                csum += dp(v, -d);
                mdiff = min(mdiff, dp(v, d + 1) - dp(v, -d));
            }
            ans = min(ans, csum + mdiff);
        }
    } else {
        int cans = 0;
        for (auto& v : graph[u]) {
            cans += dp(v, d + 1);
        }
        ans = min(ans, cans);
    }
    if (!d && !arrb[u]) {
        int cans = 0;
        for (auto& v : graph[u]) {
            cans += dp(v, 0);
        }
        ans = min(ans, cans);
    }
    return ans;
}

void solve() {
    int n;
    cin >> n >> kv;
    for (int i = 0; i < n; i++) {
        cin >> arr[i];
    }
    int m;
    cin >> m;
    for (int i = 0; i < m; i++) {
        int u;
        cin >> u;
        u--;
        arrb[u] = true;
    }
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    pdfs(0);
    memset(memo, -1, sizeof(memo));
    dbg(dp(1, -1));
    cout << dp(0, 0) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}