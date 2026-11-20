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

vector<int> facts, ans;
vector<vector<pair<int, int>>> graph;
vector<pair<int, int>> fact;
set<pair<int, int>> vis;

void dfs(int ind, int x) {
    if (ind == sz(fact)) {
        facts.push_back(x);
        return;
    }
    dfs(ind + 1, x);
    auto& [p, e] = fact[ind];
    for (int i = 0; i < e; i++) {
        x *= p;
        dfs(ind + 1, x);
    }
}

void dfs2(int ind, int s) {
    ans.push_back(s + facts[ind] - 1);
    if (!vis.emplace(ind, s).second) {
        return;
    }
    for (auto& [v, w] : graph[ind]) {
        dfs2(v, s + w - 1);
    }
}

void solve() {
    int n;
    cin >> n;
    if (n == 1) {
        cout << "1\n"
                "0" << endl;
        return;
    }
    ([&](int n) -> void {
        for (int i = 2; i < int(1e5); i++) {
            int e = 0;
            while (n % i == 0) {
                n /= i;
                e++;
            }
            if (e) {
                fact.emplace_back(i, e);
            }
        }
    })(n);
    dfs(0, 1);
    sort(begin(facts), end(facts));
    facts.erase(facts.begin());
    int m = sz(facts);
    graph.resize(m);
    for (int i = 0; i < m; i++) {
        for (int j = 0; j < i; j++) {
            if (facts[i] % facts[j] == 0) {
                int nj = int(
                    lower_bound(begin(facts), end(facts), facts[i] / facts[j]) -
                    facts.begin());
                graph[i].emplace_back(nj, facts[j]);
            }
        }
    }
    dfs2(m - 1, 0);
    sort(begin(ans), end(ans));
    ans.erase(unique(begin(ans), end(ans)), ans.end());
    cout << sz(ans) << endl;
    for (int i = 0; i < sz(ans); i++) {
        cout << ans[i] << " \n"[i == sz(ans) - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}