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

namespace brute {

int n, k, ans, arr[5][2];
vector<int> cur;
set<vector<int>> vis;

void dfs(int ind) {
    if (ind == n) {
        bool cans = false;
        if (!k) {
            cans = vis.count(cur);
        } else if (k == 1) {
            for (int i = 0; i < n; i++) {
                cur[i]++;
                cans |= vis.count(cur);
                cur[i]--;
            }
        } else {
            for (int i = 0; i < n; i++) {
                for (int j = 0; j < n; j++) {
                    cur[i]++;
                    cur[j]++;
                    cans |= vis.count(cur);
                    cur[i]--;
                    cur[j]--;
                }
            }
        }
        ans += cans;
        return;
    }
    auto& [l, r] = arr[ind];
    for (int i = l; i <= r; i++) {
        cur.push_back(i);
        dfs(ind + 1);
        cur.pop_back();
    }
}

void solve(int _n, int _k) {
    n = _n;
    k = _k;
    for (int i = 0; i < n; i++) {
        cin >> arr[i][0] >> arr[i][1];
    }
    vector<vector<int>> q;
    auto push = [&](const vector<int>& arr) -> void {
        if (vis.insert(arr).second) {
            q.push_back(arr);
        }
    };
    push(vector<int>(n));
    while (sz(q)) {
        auto u = q.back();
        q.pop_back();
        int csum = accumulate(begin(u), end(u), 0);
        for (auto& a : u) {
            for (int i = 2; i + csum <= 27; i++) {
                a += i;
                push(u);
                a -= i;
            }
        }
        for (int l = 0; l < n; l++) {
            for (int r = l + 2; r < n && r - l + 1 + csum <= 27; r++) {
                for (int i = l; i <= r; i++) {
                    u[i]++;
                }
                push(u);
                for (int i = l; i <= r; i++) {
                    u[i]--;
                }
            }
        }
    }
    ans = 0;
    dfs(0);
    cout << ans << endl;
}

}  // namespace brute

void solve() {
    int n, k;
    cin >> n >> k;
    if (n <= 5 && k <= 2) {
        brute::solve(n, k);
        return;
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}