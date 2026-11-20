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

namespace s1 {

constexpr int maxn = 1e5 + 5;

int siz[maxn];
vector<int> graph[maxn];
multiset<int> arr[maxn];

void pdfs(int u) {
    siz[u] = 1;
    for (auto& v : graph[u]) {
        pdfs(v);
        siz[u] += siz[v];
    }
}

multiset<int> dfs(int u) {
    multiset<int> cv = arr[u];
    for (auto& v : graph[u]) {
        auto&& cx = dfs(v);
        if (sz(cx) > sz(cv)) {
            swap(cv, cx);
        }
        cv.insert(begin(cx), end(cx));
    }
    while (sz(cv) > siz[u]) {
        cv.erase(cv.begin());
    }
    return cv;
}

long query_answer() {
    auto&& cv = dfs(0);
    long ans = 0;
    for (auto& a : cv) {
        ans += a;
    }
    return ans;
}

void solve() {
    int n, m, q;
    cin >> n >> m >> q;
    for (int i = 1; i < n; i++) {
        int p;
        cin >> p;
        p--;
        graph[p].push_back(i);
    }
    vector<pair<int, int>> emp(m);
    for (auto& [u, v] : emp) {
        cin >> u >> v;
        u--;
    }
    for (auto& [u, v] : emp) {
        arr[u].insert(v);
    }
    pdfs(0);
    cout << query_answer();
    while (q--) {
        int ty;
        cin >> ty;
        if (ty == 1) {
            int u, v;
            cin >> u >> v;
            u--;
            emp.emplace_back(u, v);
            arr[u].insert(v);
        } else {
            int ind;
            cin >> ind;
            ind--;
            auto& [u, v] = emp[ind];
            arr[u].erase(arr[u].find(v));
        }
        cout << " " << query_answer();
    }
    cout << endl;
}

}  // namespace s1

void solve() {
    int sid;
    cin >> sid;
    // if (sid <= 7) {
        s1::solve();
    // }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}