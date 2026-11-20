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

using V = vector<array<int, 3>>;

namespace brute {

int n, ans;
vector<int> mn, carr;
V arr;

void dfs(int ind) {
    if (ind == n) {
        for (auto& [l, r, x] : arr) {
            if (*min_element(carr.begin() + l, carr.begin() + r + 1) != x ||
                carr[l] != x) {
                return;
            }
        }
        int cans = 0;
        for (int i = 0; i < n; i++) {
            for (int j = i + 1; j < n; j++) {
                cans += carr[i] > carr[j];
            }
        }
        ans = min(ans, cans);
        return;
    }
    for (int i = mn[ind]; i < 7; i++) {
        carr.push_back(i);
        dfs(ind + 1);
        carr.pop_back();
    }
}

void solve(int _n, V _arr) {
    n = _n;
    mn = vector<int>(n, 0);
    arr = _arr;
    for (auto& [l, r, x] : arr) {
        for (int i = l; i <= r; i++) {
            mn[i] = max(mn[i], x);
        }
    }
    ans = 1e9;
    dfs(0);
    if (ans == 1e9) {
        cout << -1 << endl;
    } else {
        cout << ans << endl;
    }
}

}  // namespace brute

void solve() {
    int n, m;
    cin >> n >> m;
    V arr(m);
    for (auto& [l, r, x] : arr) {
        cin >> l >> r >> x;
        l--;
        r--;
    }
    vector<int> comp;
    for (auto& a : arr) {
        comp.push_back(a[2]);
    }
    sort(begin(comp), end(comp));
    comp.erase(unique(begin(comp), end(comp)), comp.end());
    for (auto& a : arr) {
        a[2] = int(lower_bound(begin(comp), end(comp), a[2]) - comp.begin());
    }
    brute::solve(n, arr);
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