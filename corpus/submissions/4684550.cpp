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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

bool valid(int n, int arr[], bool wtf) {
    int x = -1;
    for (int i = 0; i < n; i++) {
        if (wtf || arr[i] <= n) {
            int cx = (arr[i] - i + n) % n;
            if (x == -1) {
                x = cx;
            } else if (x != cx) {
                return false;
            }
        }
    }
    return true;
}

extern "C" int valid(int n, int arr[]) {
    return valid(n, arr, true);
}

extern "C" int replacement(int n, int arr[], int out[]) {
    int targ[n];
    int x = 0;
    for (int i = 0; i < n; i++) {
        if (arr[i] <= n) {
            x = arr[i] - i;
        }
    }
    for (int i = 0; i < n; i++) {
        targ[i] = (x + i + n) % n;
        if (!targ[i]) {
            targ[i] = n;
        }
        dbg(targ[i]);
    }
    map<int, int> rev;
    for (int i = 0; i < n; i++) {
        rev[arr[i]] = i;
    }
    vector<int> ans;
    int mind = max_element(arr, arr + n) - arr, e = arr[mind];
    for (int i = n + 1; i <= e; i++) {
        auto it = rev.find(i);
        int cind;
        if (it == rev.end()) {
            cind = mind;
        } else {
            cind = it->second;
        }
        ans.push_back(targ[cind]);
        targ[cind] = i;
    }
    copy(begin(ans), end(ans), out);
    return sz(ans);
}

const long mod = 1e9 + 9;

long bpow(long base, long exp) {
    long ans = 1;
    while (exp) {
        if (exp & 1) {
            ans = (ans * base) % mod;
        }
        base = (base * base) % mod;
        exp >>= 1;
    }
    return ans;
}

extern "C" int countReplacement(int n, int arr[]) {
    if (!valid(n, arr, false)) {
        return 0;
    }
    sort(arr, arr + n);
    long ans = 1;
    if (arr[0] > n) {
        ans = n;
    }
    vector<int> cur {n};
    for (int i = 0; i < n; i++) {
        if (arr[i] > n) {
            cur.push_back(arr[i]);
        }
    }
    for (int i = 0; i < sz(cur) - 1; i++) {
        ans = (ans * bpow(sz(cur) - i - 1, cur[i + 1] - cur[i] - 1)) % mod;
        dbg(ans);
    }
    return ans;
}