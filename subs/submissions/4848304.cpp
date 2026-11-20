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

int GetBestPosition(int n, int m, int kv, int *arr, int *ql, int *qr) {
    pair<int, int> ans {};
    for (int i = 0; i < n; i++) {
        vector<int> carr(arr, arr + n - 1);
        carr.insert(carr.begin() + i, kv);
        int cans = 0;
        for (int j = 0; j < m; j++) {
            auto l = carr.begin() + ql[j], r = carr.begin() + qr[j] + 1;
            int mx = *max_element(l, r);
            carr.erase(l, r);
            carr.insert(carr.begin() + ql[j], mx);
            cans += mx == kv;
        }
        ans = max(ans, pair<int, int> {cans, i});
    }
    return ans.second;
}