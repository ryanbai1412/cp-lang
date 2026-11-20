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
	dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
	int n;
	long cl, cr, y;
	cin >> n >> cl >> cr >> y;
	vector<pair<long, int>> evts;
	for (int i = 0; i < n; i++) {
		long x, v, h;
		cin >> x >> v >> h;
		long add = h * y / v;
		if (h * y % v == 0) {
			add--;
		}
		long l = x - add, r = x + add;
		dbg(l, r, h * y / v);
		if (l <= r) {
			evts.emplace_back(l, +1);
			evts.emplace_back(r + 1, -1);
		}
	}
	long ans[n + 1] {};
	sort(begin(evts), end(evts));
	long prev = -1e18;
	int cur = 0;
	for (int i = 0; i < sz(evts); ) {
		long start = evts[i].first;
		long l = max(cl, prev), r = min(cr + 1, start);
		dbg(cur, l, r);
		ans[cur] += max(long(0), r - l);
		prev = start;
		for (; i < sz(evts) && evts[i].first == start; i++) {
			cur += evts[i].second;
		}
	}
	assert(!cur);
	ans[0] += max(long(0), cr - prev + 1);
	for (int i = 0; i < n; i++) {
		ans[i + 1] += ans[i];
	}
	for (auto& a : ans) {
		cout << a << endl;
	}
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}