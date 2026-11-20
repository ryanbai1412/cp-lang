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

namespace __gnu_pbds {
template <typename T, typename U = less<T>>
using ost =
    tree<T, null_type, U, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

void solve() {
    int n, kv;
    long m;
    cin >> n >> kv >> m;
    long arr[2][n];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    map<long, ost<int>> mp;
    for (int i = 0; i < n; i++) {
        mp[arr[0][i]].insert(i);
    }
    long ans = 0;
    for (int i = 0; i < n; i++) {
        auto it = mp.find(m - arr[1][i]);
        if (it == mp.end()) {
            continue;
        }
        auto& s = it->second;
        ans += int(s.order_of_key(i - kv + 1));
        ans += sz(s) - int(s.order_of_key(i + kv));
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}