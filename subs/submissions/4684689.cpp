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

vector<int> find_subset(int ql, int qr, vector<int> carr) {
    int n = sz(carr);
    vector<pair<int, int>> arr;
    for (int i = 0; i < n; i++) {
        arr.emplace_back(carr[i], i);
    }
    sort(begin(arr), end(arr));
    long psum = 0, ssum = 0;
    for (int l = 0, r = n - 1; l < n; l++, r--) {
        psum += arr[l].first;
        ssum += arr[r].first;
        if (qr < psum || ssum < ql) {
            continue;
        }
        vector<int> cur;
        for (int i = 0; i <= l; i++) {
            cur.push_back(i);
        }
        for (int i = l; i >= 0 && psum < ql; i--) {
            int nxt = i + n - l - 1;
            cur[i] = nxt;
            psum += arr[nxt].first - arr[i].first;
        }
        for (auto& a : cur) {
            a = arr[a].second;
        }
        return cur;
    }
    return {};
}