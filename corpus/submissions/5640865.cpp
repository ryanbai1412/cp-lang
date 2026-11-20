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

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int maxn = 5e5 + 5;

int r1[maxn];
vector<pair<int, int>> ans;
vector<int> graph[maxn];

void pdfs(int u, int p, int r) {
    if (r == -1 && p != -1) {
        r = u;
    }
    r1[u] = r;

    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        }
        pdfs(v, u, r);
    }
}

vector<int> dfs(int u, int p) {
    if (sz(graph[u]) == 1) {
        return {u};
    }

    vector<int> unmatched;
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        }

        auto ch = dfs(v, u);

        while (sz(unmatched) > 1 && sz(ch)) {
            ans.emplace_back(unmatched.back(), ch.back());
            unmatched.pop_back();
            ch.pop_back();
        }

        unmatched.insert(unmatched.end(), begin(ch), end(ch));
        assert(sz(unmatched) <= 3);
    }

    if (sz(unmatched) == 3) {
        ans.emplace_back(unmatched[0], unmatched[1]);
        unmatched = {unmatched[2]};
    }

    if (p == -1 && sz(unmatched)) {
        if (sz(unmatched) == 2) {
            ans.emplace_back(unmatched[0], unmatched[1]);
        } else {
            assert(sz(unmatched) == 1);
            int left = unmatched[0];
            for (auto& v : graph[u]) {
                assert(v == r1[v]);
                if (v != r1[left]) {
                    ans.emplace_back(v, left);
                    unmatched.pop_back();
                    break;
                }
            }
            assert(!sz(unmatched));
        }
    }

    return unmatched;
}

void solve() {
    int n;
    cin >> n;
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }

    for (int i = 0; i < n; i++) {
        if (sz(graph[i]) < 2) {
            continue;
        }
        pdfs(i, -1, -1);
        dfs(i, -1);
        break;
    }

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