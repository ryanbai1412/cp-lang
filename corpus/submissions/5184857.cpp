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

struct DS {
    bool bad = false;
    map<char, char> mp;

    void insert(char a, char b) {
        auto [it, _] = mp.emplace(a, b);
        bad |= it->second != b;
    }

    char operator[](char c) {
        auto it = mp.find(c);
        if (it == mp.end()) {
            bad = true;
            return '?';
        }
        return it->second;
    }
};

void solve() {
    string enc, dec, s;
    cin >> enc >> dec >> s;
    DS e2d, d2e;
    for (int i = 0; i < sz(enc); i++) {
        e2d.insert(enc[i], dec[i]);
        d2e.insert(dec[i], enc[i]);
    }
    string ans;
    for (auto& a : s) {
        ans += e2d[a];
    }
    if (e2d.bad || d2e.bad || sz(d2e.mp) != 26) {
        cout << "Failed" << endl;
    } else {
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}