#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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
    int n;
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }

    map<long, int> cnt;
    for (auto& a : arr) {
        cnt[a]++;
    }

    int q;
    cin >> q;
    long radd = 0, csum = 0;
    for (auto& a : arr) {
        csum += a;
    }
    while (q--) {
        string ty;
        cin >> ty;
        if (ty == "INFLATION") {
            int x;
            cin >> x;
            radd += x;
        } else {
            long u, v;
            cin >> u >> v;
            if (u != v) {
                int &ccnt = cnt[u - radd];
                cnt[v - radd] += ccnt;
                csum += (v - u) * ccnt;
                ccnt = 0;
            }
        }
        cout << csum + radd * n << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}