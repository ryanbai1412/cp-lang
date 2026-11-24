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
 
using ll = long long;
 
#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())
 
// The original calls the IOI grader's allocate_tickets. This stand-in keeps the allocation,
// and `main` reads n m k and the tickets, then prints the prize and the allocation.
static std::vector<std::vector<int>> allocated;
void allocate_tickets(std::vector<std::vector<int>> s) { allocated = s; }
 
ll find_maximum(int k, vector<vector<int>> arr) {
    int n = sz(arr), m = sz(arr[0]);
    long ans = 0;
    for (auto& a : arr) {
        assert(is_sorted(begin(a), end(a)));
        ans -= accumulate(a.begin(), a.begin() + k, long(0));
    }
    int cnt[n] {};
    priority_queue<pair<int, int>> pq;
    auto upd = [&](int i) -> void {
        if (cnt[i] >= k) {
            return;
        }
        pq.emplace(arr[i][k - cnt[i] - 1] + arr[i][m - cnt[i] - 1], i);
    };
    for (int i = 0; i < n; i++) {
        upd(i);
    }
    for (int i = 0; i < n * k / 2; i++) {
        auto [d, j] = pq.top();
        pq.pop();
        ans += d;
        cnt[j]++;
        upd(j);
    }
    vector<vector<int>> qans(n, vector<int>(m, -1));
    vector<int> inds[n][2];
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < k - cnt[i]; j++) {
            inds[i][0].push_back(j);
        }
        for (int j = 0; j < cnt[i]; j++) {
            inds[i][1].push_back(m - j - 1);
        }
    }
    for (int i = 0; i < k; i++) {
        int cur = 0;
        auto add = [&](int j, int ci) -> void {
            cur += !ci;
            qans[j][inds[j][ci].back()] = i;
            inds[j][ci].pop_back();
        };
        vector<int> v;
        for (int j = 0; j < n; j++) {
            if (!sz(inds[j][0])) {
                add(j, 1);
            } else if (!sz(inds[j][1])) {
                add(j, 0);
            } else {
                v.push_back(j);
            }
        }
        assert(cur <= n / 2);
        for (auto& a : v) {
            if (cur < n / 2) {
                add(a, 0);
            } else {
                add(a, 1);
            }
        }
    }
    allocate_tickets(qans);
    return ans;
}
int main() {
	int n, m, k;
	if (scanf("%d %d %d", &n, &m, &k) != 3) return 1;
	vector<vector<int>> x(n, vector<int>(m));
	for (auto& row : x)
		for (auto& v : row) scanf("%d", &v);
	cout << find_maximum(k, x) << "\n";
	for (auto& row : allocated)
		for (int j = 0; j < m; j++) printf("%d%c", row[j], j + 1 < m ? ' ' : '\n');
}
