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
    int p1, p2, p3;
    cin >> p1 >> p2 >> p3;
    string s;
    cin >> s;
    string ans;
    for (int i = 0; i < sz(s); i++) {
        if (s[i] == '-' && 1 <= i && i + 1 < sz(s)) {
            char l = s[i - 1], r = s[i + 1];
            if (l < r && ((isdigit(l) && isdigit(r)) || (islower(l) && islower(r)))) {
                string cs;
                for (char c = l + 1; c < r; c++) {
                    cs += c;
                }
                if (p1 == 1) {
                    for (auto& a : cs) {
                        a = tolower(a);
                    }
                } else if (p1 == 2) {
                    for (auto& a : cs) {
                        a = toupper(a);
                    }
                } else if (p1 == 3) {
                    for (auto& a : cs) {
                        a = '*';
                    }
                } else {
                    assert(false);
                }
                {
                    string ns;
                    for (auto& a : cs) {
                        for (int j = 0; j < p2; j++) {
                            ns += a;
                        }
                    }
                    cs = ns;
                }
                if (p3 == 2) {
                    reverse(begin(cs), end(cs));
                }
                ans += cs;
                continue;
            }
        }
        ans += s[i];
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}