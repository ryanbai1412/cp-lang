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
    int arra[n], arrb[m];
    for (auto& a : arra) {
        cin >> a;
    }
    for (auto& a : arrb) {
        cin >> a;
    }
    map<int, int> cnt;
    for (int i = 0; i < n; i++) {
        for (int j = i + 1; j < n; j++) {
            cnt[arra[j] - arra[i]]++;
        }
    }
    long ans = 0;
    for (int i = 0; i < m; i++) {
        for (int j = i + 1; j < m; j++) {
            ans += cnt[arrb[j] - arrb[i]];
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