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
    double kv;
    int n;
    cin >> kv >> n;
    double arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    double p = arr[0], t = 0;
    for (auto a : arr) {
        if (abs(p + kv - a) <= t) {
            p += kv;
            continue;
        }
        if (a <= p + kv) {
            a += t;
            p = a;
            continue;
        }
        a -= t;
        double nt = (a - p - kv) / 2;
        a -= nt;
        t += nt;
        p = a;
    }
    cout << t << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(3);
    solve();
}