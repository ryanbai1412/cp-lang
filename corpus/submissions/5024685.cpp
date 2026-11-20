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
    string iarr[3];
    for (auto& a : iarr) {
        cin >> a;
    }
    set<char> s;
    for (auto& a : iarr) {
        for (auto& b : a) {
            s.insert(b);
        }
    }
    assert(sz(s) <= 10);
    int cs[26];
    {
        int ind = 0;
        for (auto& a : s) {
            cs[a - 'A'] = ind++;
        }
    }
    vector<int> arr[3], first;
    for (int i = 0; i < 3; i++) {
        for (auto& a : iarr[i]) {
            arr[i].push_back(cs[a - 'A']);
        }
        first.push_back(arr[i][0]);
        arr[i].insert(arr[i].begin(), 22 - sz(arr[i]), -1);
    }
    int perm[10];
    iota(begin(perm), end(perm), 0);
    do {
        auto get = [&](int x) -> int {
            return x == -1 ? 0 : perm[x];
        };
        int sum[22];
        for (int i = 0; i < 22; i++) {
            sum[i] = get(arr[0][i]);
        }
        for (int i = 21, carry = 0; i >= 0; i--) {
            sum[i] += get(arr[1][i]) + carry;
            carry = sum[i] >= 10;
            sum[i] %= 10;
        }
        for (int i = 0; i < 22; i++) {
            if (get(arr[2][i]) != sum[i]) {
                goto loop;
            }
        }
        for (auto& a : first) {
            if (!get(a)) {
                goto loop;
            }
        }
        for (auto& a : iarr) {
            for (auto& b : a) {
                cout << perm[cs[b - 'A']];
            }
            cout << endl;
        }
        return;
    loop:;
    } while (next_permutation(begin(perm), end(perm)));
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}