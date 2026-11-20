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
    string s;
    cin >> s;
    map<char, int> mp;
    for (auto& a : s) {
        assert(islower(a));
        mp[a]++;
    }
    vector<int> v;
    for (auto& [_, cnt] : mp) {
        v.push_back(cnt);
    }
    sort(begin(v), end(v));
    int x = v.back() - v[0];
    bool prime = x >= 2;
    for (int i = 2; i < x; i++) {
        prime &= x % i == 0;
    }
    if (prime) {
        cout << "Lucky Word" << endl;
        cout << x << endl;
    } else {
        cout << "No Answer" << endl;
        cout << x << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}