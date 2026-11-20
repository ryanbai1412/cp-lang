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
    int n, m;
    cin >> n >> m;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int arrw[m];
    for (auto& a : arrw) {
        cin >> a;
    }
    set<int> spots;
    queue<int> q;
    for (int i = 0; i < n; i++) {
        spots.insert(i);
    }
    int car_loc[m];
    long ans = 0;
    for (int _it = 0; _it < 2 * m; _it++) {
        int x;
        cin >> x;
        int ind = abs(x) - 1;
        if (x < 0) {
            spots.insert(car_loc[ind]);
        } else {
            q.push(ind);
        }
        while (sz(q) && sz(spots)) {
            int u = q.front();
            dbg(u);
            q.pop();
            auto it = spots.begin();
            car_loc[u] = *it;
            ans += arr[*it] * arrw[u];
            spots.erase(it);
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