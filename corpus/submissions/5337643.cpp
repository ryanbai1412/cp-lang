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

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

void solve() {
    int n;
    long a, b;
    cin >> n >> a >> b;
    long ans = 0;
    min_pq<pair<long, int>> pq;
    int rows[n] {};
    for (int i = 0; i < n; i++) {
        pq.emplace(a * (i + 1) + b, i);
    }
    for (int _iter = 0; _iter < n; _iter++) {
        auto [w, i] = pq.top();
        pq.pop();
        ans += w;
        pq.emplace(a * (i + 1) + b * (++rows[i] + 1), i);
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}