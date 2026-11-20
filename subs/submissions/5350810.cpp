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

#define int long

template <typename T>
T cdiv(T a, T b) {
    return (a + b - 1) / b;
}

bool check(int n, int x) {
    set<int> vis;
    while (true) {
        int d = x / n;
        x = (x % n) * 3;
        if (d == 1 && x) {
            return false;
        } else if (!vis.insert(x).second) {
            break;
        }
    }
    return true;
}

int n;
vector<int> ans;

void dfs(int l, int r, int k, int b) {
    if (r - l <= 10) {
        for (int i = l; i <= r; i++) {
            if (check(n, i)) {
                ans.push_back(i);
            }
        }
        return;
    }
    dfs(l, int(long(n) * (3 * k + 1) / (3 * b)), 3 * k, 3 * b);
    dfs(int(cdiv(long(3 * k + 2) * n, long(3 * b))), r, 3 * k + 2, 3 * b);
}

void solve() {
    cin >> n;
    dfs(0, n, 0, 1);
    for (auto& a : ans) {
        cout << a << endl;
    }
}

int32_t main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}