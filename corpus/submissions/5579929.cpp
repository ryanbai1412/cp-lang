#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

string complement(string s) {
    for (auto& a : s) {
        if (a == '0') {
            a = '1';
        } else if (a == '1') {
            a = '0';
        } else {
            assert(false);
        }
    }
    return s;
}

int solve0(string s) {
    if (count(begin(s), end(s), '0')) {
        return -1;
    }
    return 0;
}

int solve00(string s) {
    int c0 = int(count(begin(s), end(s), '0')),
        c1 = int(count(begin(s), end(s), '1'));
    if (c0 > c1 + 1) {
        return -1;
    }
    int ans = 0;
    for (int i = 0; i < sz(s) - 1; i++) {
        ans += s[i + 1] == '0' && s[i] == '0';
    }
    return ans;
}

int solve01(string s) {
    int ans = 0;
    for (int i = 0; i < sz(s) - 1; i++) {
        ans += s[i + 1] == '1' && s[i] == '0';
    }
    return ans;
}

int solve010(string s) {
    int ans = 0;
    for (int i = 0; i < sz(s) - 2; i++) {
        ans += s.substr(i, 3) == "010";
    }
    return (ans + 1) / 2;
}

int solve001(string s) {
    int ans = 0;
    for (int i = 0; i < sz(s) - 2; i++) {
        ans += s.substr(i, 3) == "001";
    }
    return ans;
}

int solve011(string s) {
    int ans = 0;
    for (int i = 0; i < sz(s) - 2; i++) {
        ans += s.substr(i, 3) == "011";
    }
    return ans;
}

int solve000(string s) {
    int c0 = int(count(begin(s), end(s), '0')),
        c1 = int(count(begin(s), end(s), '1'));
    if (c0 > 2 * (c1 + 1)) {
        return -1;
    }
    int n = sz(s), ans = 0;
    vector<int> ones {-1};
    for (int i = 0; i < n; i++) {
        if (s[i] == '1') {
            ones.push_back(i);
        }
    }
    ones.push_back(n);
    vector<int> vals;
    for (int i = 0; i < sz(ones) - 1; i++) {
        vals.push_back(ones[i + 1] - ones[i] - 1);
    }
    int cnt0 = 0, cnt1 = 0;
    for (auto& a : vals) {
        if (a == 0) {
            cnt0++;
        } else if (a == 1) {
            cnt1++;
        }
    }
    for (auto& a : vals) {
        while (a > 2) {
            if (a == 3) {
                if (cnt1) {
                    cnt1--;
                    ans++;
                } else {
                    assert(cnt0);
                    cnt0--;
                    cnt1++;
                    ans++;
                }
                a--;
            } else {
                if (cnt0) {
                    cnt0--;
                    a -= 2;
                    ans++;
                } else {
                    assert(cnt1);
                    cnt1--;
                    a--;
                    ans++;
                }
            }
        }
    }
    return ans;
}

void solve() {
    string s, t;
    cin >> s >> t;
    string cs = complement(s);
    if (t == "0") {
        cout << solve0(s) << endl;
    } else if (t == "1") {
        cout << solve0(cs) << endl;
    } else if (t == "00") {
        cout << solve00(s) << endl;
    } else if (t == "11") {
        cout << solve00(cs) << endl;
    } else if (t == "01") {
        cout << solve01(s) << endl;
    } else if (t == "10") {
        cout << solve01(cs) << endl;
    } else if (t == "010") {
        cout << solve010(s) << endl;
    } else if (t == "101") {
        cout << solve010(cs) << endl;
    } else if (t == "001") {
        cout << solve001(s) << endl;
    } else if (t == "110") {
        cout << solve001(cs) << endl;
    } else if (t == "011") {
        cout << solve011(s) << endl;
    } else if (t == "100") {
        cout << solve011(cs) << endl;
    } else if (t == "000") {
        cout << solve000(s) << endl;
    } else if (t == "111") {
        cout << solve000(cs) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
    // int tcs;
    // cin >> tcs;
    // while (tcs--) {
    //     solve();
    // }
}