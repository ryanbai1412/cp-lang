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

int fdiv(int a, int b) {
    assert(b >= 0);
    if (a >= 0) {
        return a / b;
    } else {
        return (a - b + 1) / b;
    }
}

int cdiv(int a, int b) {
    return fdiv(a + b - 1, b);
}

void solve() {
    int x, base;
    cin >> x >> base;
    base = -base;
    vector<int> ans;
    bool neg = false;
    while (x) {
        if (neg) {
            int nx = cdiv(x, base);
            ans.push_back(nx * base - x);
            x = nx;
        } else {
            int nx = fdiv(x, base);
            ans.push_back(x - nx * base);
            x = nx;
        }
        neg ^= true;
    }
    reverse(begin(ans), end(ans));
    for (auto& a : ans) {
        if (a < 10) {
            cout << a;
        } else {
            cout << char('A' + a - 10);
        }
    }
    cout << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}