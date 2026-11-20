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

void solve() {
    int m, n;
    cin >> m >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int ans = 0;
    set<int> vis;
    deque<int> q;
    for (auto& a : arr) {
        if (vis.count(a)) {
            continue;
        }
        ans++;
        q.push_back(a);
        vis.insert(a);
        while (sz(q) > m) {
            vis.erase(q.front());
            q.pop_front();
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}