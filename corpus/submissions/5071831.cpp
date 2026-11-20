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

struct DS {
    priority_queue<pair<int, int>> v;

    void insert(int cnt) {
        v.emplace(0, cnt);
    }

    void merge(DS&& ds, int w) {
        while (sz(ds.v)) {
            auto [x, cnt] = ds.v.top();
            ds.v.pop();
            v.emplace(x + w, cnt);
        }
    }

    long query(int kv) {
        int used = 0;
        long ans = 0;
        while (used < kv && sz(v)) {
            auto [x, cnt] = v.top();
            v.pop();
            int cur = min(cnt, kv - used);
            ans += long(x) * cur;
            used += cur;
            if (cur < cnt) {
                v.emplace(x, cnt - cur);
            }
        }
        v.emplace(0, used);
        return ans;
    }
};

void solve() {
    int n;
    cin >> n;
    int sarr[n], earr[n];
    for (auto& a : sarr) {
        cin >> a;
    }
    for (auto& a : earr) {
        cin >> a;
    }
    vector<pair<int, int>> graph[n];
    for (int i = 1; i < n; i++) {
        int p, w;
        cin >> p >> w;
        p--;
        graph[p].emplace_back(i, w);
    }
    long ans = 0;
    DS dp[n];
    for (int i = n - 1; i >= 0; i--) {
        for (auto& [v, w] : graph[i]) {
            dp[i].merge(std::move(dp[v]), w);
        }
        ans += dp[i].query(sarr[i]);
        dp[i].insert(earr[i]);
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}