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
#define sz(x) int((x).size())

void solve() {
	int base, queries, n, m;
	cin >> base >> queries >> n >> m;
	auto md = [&](long x) -> int {
		x %= base;
		if (x < 0) {
			x += base;
		}
		return int(x);
	};
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	vector<int> vals[base];
	for (auto& a : arr) {
		vals[md(a)].push_back(a);
	}
	while (queries--) {
		long x;
		cin >> x;
		map<long, int> from;
		vector<long> q = {x};
		from[x] = 0;
		for (int i = 0; i < 100; i++) {
			vector<long> nq;
			for (auto& a : q) {
				for (auto& b : vals[md(a)]) {
					long nxt = (a - b) / base;
					if (from.emplace(nxt, b).second) {
						nq.push_back(nxt);
					}
				}
			}
			if (from.count(0)) {
				long u = 0;
				vector<int> ans;
				while (u != x) {
					int cx = from[u];
					ans.push_back(cx);
					u = u * base + cx;
				}
				for (int j = 0; j < sz(ans); j++) {
					cout << ans[j] << " \n"[j == sz(ans) - 1];
				}
				goto loop;
			}
			swap(q, nq);
		}
		cout << "IMPOSSIBLE" << endl;
	loop:;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}