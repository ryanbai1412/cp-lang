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
    int q;
    cin >> q;
    int mx = -1;
    set<int> good;
    int pans = 0;
    while (q--) {
        int x;
        cin >> x;
        x = (x + pans) % int(1e9);
        if (x > mx) {
            mx = x;
            good.insert(x);
        }
        while (sz(good) && *(--good.end()) > x) {
            good.erase(--good.end());
        }
        pans = sz(good);
        cout << pans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}