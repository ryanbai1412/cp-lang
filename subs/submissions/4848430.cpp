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

constexpr int maxn = 2e5 + 5, bsize = 500;

int arr[maxn], tin[maxn], tout[maxn];
vector<int> ord, graph[maxn];

struct Solver {
    int kv;
    long ans[maxn] {};

    Solver(int kv) : kv(kv) {
        dfs(0);
    }

    void dfs(int u, int p = 0) {
        ans[arr[u]] += p;
        p += arr[u] == kv;
        for (auto& v : graph[u]) {
            dfs(v, p);
        }
    }

    long operator[](int i) const {
        return ans[i];
    }
};

void pdfs(int u) {
    tin[u] = sz(ord);
    ord.push_back(u);
    for (auto& v : graph[u]) {
        pdfs(v);
    }
    tout[u] = sz(ord);
}

void solve() {
    int n, m, q;
    cin >> n >> m >> q;
    for (int i = 0; i < n; i++) {
        if (i) {
            int p;
            cin >> p;
            p--;
            graph[p].push_back(i);
        }
        cin >> arr[i];
        arr[i]--;
    }
    pdfs(0);
    vector<int> inds[m], oinds[m];
    for (int i = 0; i < n; i++) {
        inds[arr[i]].push_back(i);
        oinds[arr[ord[i]]].push_back(i);
    }
    Solver* opts[m] {};
    for (int i = 0; i < m; i++) {
        if (sz(inds[i]) > bsize) {
            opts[i] = new Solver(i);
        }
    }
    auto cnt_range = [&](int l, int r, int x) -> int {
        return int(lower_bound(begin(oinds[x]), end(oinds[x]), r) -
                   lower_bound(begin(oinds[x]), end(oinds[x]), l));
    };
    auto solve = [&](int r1, int r2) -> long {
        auto inner = [&](int r1, int r2) -> long {
            static map<pair<int, int>, long> cache;
            auto [it, inserted] = cache.insert({{r1, r2}, 0});
            if (!inserted) {
                return it->second;
            }
            long& ans = it->second;
            for (auto& a : inds[r1]) {
                ans += cnt_range(tin[a], tout[a], r2);
            }
            return ans;
        };
        if (opts[r1]) {
            return (*opts[r1])[r2];
        }
        return inner(r1, r2);
    };
    while (q--) {
        int r1, r2;
        cin >> r1 >> r2;
        r1--;
        r2--;
        cout << solve(r1, r2) << endl;
    }
    for (auto& a : opts) {
        if (a) {
            delete a;
        }
    }
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}