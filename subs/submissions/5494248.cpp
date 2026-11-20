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
    int n, q;
    cin >> q >> n;
    set<pair<int, int>> s;
    while (q--) {
        int ty;
        cin >> ty;
        if (ty == 1) {
            cout << sz(s) << endl;
            continue;
        }
        int l, r;
        cin >> l >> r;
        auto it = s.lower_bound({l, -1});
        int cnt = 0;
        while (it != s.end() && it->first <= r) {
            it = s.erase(it);
            cnt++;
        }
        if (it != s.begin() && l <= (--it)->second) {
            s.erase(it);
            cnt++;
        }
        s.emplace(l, r);
        cout << cnt << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}