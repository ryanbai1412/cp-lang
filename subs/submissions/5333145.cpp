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
    int kv;
    string s;
    cin >> kv >> s;
    int emp = 0;
    for (auto& a : s) {
        if (a == '*') {
            emp++;
        } else {
            kv -= a - 'a' + 1;
        }
    }
    if (kv < emp || kv > emp * 26) {
        cout << "Impossible" << endl;
        return;
    }
    vector<int> ans;
    for (int i = 0; i < emp; i++) {
        int csub = min(kv - (emp - 1 - i), 26);
        ans.push_back(csub);
        kv -= csub;
    }
    for (auto& a : s) {
        if (a == '*') {
            a = char('a' + ans.back() - 1);
            ans.pop_back();
        }
    }
    cout << s << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}