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
#define sz(x) int(size(x))

namespace __gnu_pbds {
template <typename T, typename U = less<T>>
using ost =
    tree<T, null_type, U, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

void solve() {
    int n, m;
    cin >> n >> m;
    int arr[n][2];
    for (auto& [l, r] : arr) {
        cin >> l >> r;
    }
    vector<tuple<int, bool, int, int>> evts;
    for (int i = 0; i < n; i++) {
        auto& [l, r] = arr[i];
        evts.emplace_back(l, true, r - l, i);
        evts.emplace_back(r + 1, false, r - l, i);
    }
    sort(begin(evts), end(evts));
    int ans = 2e9;
    multiset<int> s;
    for (auto& [_, b, x, i] : evts) {
        if (b) {
            s.insert(x);
            if (sz(s) >= m) {
                queue<int> q;
                for (auto& a : s) {
                    q.push(a);
                    if (sz(q) >= m) {
                        ans = min(ans, a - q.front());
                        q.pop();
                    }
                }
            }
        } else {
            s.erase(s.find(x));
        }
    }
    if (ans == 2e9) {
        cout << -1 << endl;
    } else {
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}