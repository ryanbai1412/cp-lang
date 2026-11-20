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

const vector<int> primes {2,  3,  5,  7,  11, 13, 17, 19, 23, 29, 31, 37, 41,
                          43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97};

bool iprime(int x) {
    if (x < 2) {
        return false;
    }
    for (int i = 2; i < x; i++) {
        if (x % i == 0) {
            return false;
        }
    }
    return true;
}

void solve() {
    string s;
    cin >> s;
    map<char, int> mp;
    for (auto& a : s) {
        assert(islower(a) || isupper(a));
        mp[a]++;
    }
    vector<int> v;
    for (auto& [_, cnt] : mp) {
        v.push_back(cnt);
    }
    sort(begin(v), end(v));
    int x = v.back() - v[0];
    assert(iprime(x) == (find(begin(primes), end(primes), x) != primes.end()));
    if (iprime(x)) {
        cout << "Lucky Word" << endl;
        cout << x << endl;
    } else {
        cout << "No Answer" << endl;
        cout << 0 << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}