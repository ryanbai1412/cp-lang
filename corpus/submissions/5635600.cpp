#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

using i128 = __int128_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

constexpr int maxn = 20;

bool arr[maxn][maxn];
i128 memo[maxn][maxn];
vector<int> c_nodes, graph[maxn];

void pdfs(int u) {
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        pdfs(v);
    }
}

i128 dp(int u, int val) {
    i128& ans = memo[u][val];
    if (ans != -1) {
        return ans;
    }

    ans = 1;

    for (auto& v : graph[u]) {
        i128 cans = 0;

        for (auto& val2 : c_nodes) {
            if (!arr[val][val2]) {
                continue;
            }

            cans += dp(v, val2);
        }

        ans *= cans;
    }

    return ans;
}

void solve() {
    int n, m;
    cin >> n >> m;
    for (int i = 0; i < m; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        arr[u][v] = arr[v][u] = true;
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

    auto pow_n1 = [&](int x) -> i128 {
        if (x & 1) {
            return -1;
        } else {
            return 1;
        }
    };

    i128 ans = 0;

    for (int mask = 1; mask < (1 << n); mask++) {
        c_nodes.clear();
        for (int i = 0; i < n; i++) {
            if (on(mask, i)) {
                c_nodes.push_back(i);
            }
        }

        memset(memo, -1, sizeof(memo));
        for (auto& a : c_nodes) {
            ans += pow_n1(__builtin_popcount(mask) + n) * dp(0, a);
        }
    }

    cout << long(ans) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}