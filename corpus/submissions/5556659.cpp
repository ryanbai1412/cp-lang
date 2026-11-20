#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

set<int> solve1(int n, int start, const vector<pair<int, int>>& arr) {
    int m = sz(arr);
    vector<int> graph[n];
    for (int i = 0; i < m; i++) {
        graph[arr[i].first].push_back(i);
    }
    set<int> gvis, ans;
    for (int i = 0; i < n; i++) {
        gvis.insert(i);
    }
    vector<int> q;
    bool vis[m] {};
    auto push = [&](int u) -> void {
        if (vis[u]) {
            return;
        }
        vis[u] = true;
        q.push_back(u);
    };
    for (int i = 0; i < m; i++) {
        auto& [l, r] = arr[i];
        if (l <= start && start < r) {
            push(i);
        }
    }
    while (sz(q)) {
        int u = q.back();
        q.pop_back();
        auto& [l, r] = arr[u];
        ans.insert(r);
        for (auto it = gvis.lower_bound(max(start, l));
             it != gvis.end() && *it <= r; it = gvis.erase(it)) {
            for (auto& v : graph[*it]) {
                push(v);
            }
        }
    }
    ans.erase(start);
    return ans;
}

void solve() {
    int n, m, start;
    cin >> n >> m >> start;
    start--;
    vector<pair<int, int>> arr(m);
    for (auto& [l, r] : arr) {
        cin >> l >> r;
        l--;
        r--;
    }
    auto cans = solve1(n, start, arr);
    auto rans = solve1(n, n - start - 1, ({
                           vector<pair<int, int>> v = arr;
                           for (auto& [l, r] : v) {
                               l = n - l - 1;
                               r = n - r - 1;
                               swap(l, r);
                           }
                           v;
                       }));
    for (auto& a : rans) {
        cans.insert(n - a - 1);
    }
    vector<int> ans(begin(cans), end(cans));
    for (int i = 0; i < sz(ans); i++) {
        cout << ans[i] + 1 << " \n"[i == sz(ans) - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}