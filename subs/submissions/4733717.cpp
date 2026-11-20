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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
    int n, m;
    long k;
    cin >> n >> m >> k;
    k *= 2;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
        a *= 2;
    }
    for (int qi = 0; qi < m; qi++) {
        int x;
        cin >> x;
        arr.push_back(x * 2);
        sort(begin(arr), end(arr));
        auto check = [&](long x) -> bool {
            long prev = -4e18;
            for (auto& a : arr) {
                long cur = max(prev + k, a - x);
                if (abs(cur - a) > x) {
                    return false;
                }
                prev = cur;
            }
            return true;
        };
        long l = 0, r = 2e18;
        while (l < r) {
            long mid = (l + r) / 2;
            if (check(mid)) {
                r = mid;
            } else {
                l = mid + 1;
            }
        }
        cout << l / 2;
        if (l & 1) {
            cout << ".5";
        }
        cout << " \n"[qi == m - 1];
    }
}

int main() {
    cout << fixed;
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}