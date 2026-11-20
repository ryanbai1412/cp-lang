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

constexpr int maxn = 1e6 + 5;

vector<int> graph[2][maxn];
vector<pair<int, int>> ans;

void dfs(int u) {
    set<int> v0(begin(graph[0][u]), end(graph[0][u]));
    for (auto& v : graph[1][u]) {
        if (!v0.count(v)) {
            ans.emplace_back(v, u);
        }
        dfs(v);
    }
}

void solve() {
    int n;
    cin >> n;
    int root[2];
    for (int i = 0; i < 2; i++) {
        for (int j = 0; j < n; j++) {
            int p;
            cin >> p;
            if (p == -1) {
                root[i] = j;
            } else {
                p--;
                graph[i][p].push_back(j);
            }
        }
    }
    if (root[0] != root[1]) {
        cout << -1 << endl;
        return;
    }
    dfs(root[0]);
    cout << sz(ans) << endl;
    for (auto& [u, v] : ans) {
        cout << u + 1 << " " << v + 1 << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}