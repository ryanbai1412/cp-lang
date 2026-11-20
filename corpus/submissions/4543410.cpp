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

struct chash {
	uint64_t operator()(uint64_t x) const {
		x ^= x << 17;
		x ^= x >> 13;
		x ^= x << 19;
		x ^= x >> 16;
		return x;
	}
};

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
		__gnu_pbds::gp_hash_table<long, int, chash> from;
		vector<long> q = {x};
		for (int i = 0; i < 100; i++) {
			vector<long> nq;
			for (auto& a : q) {
				for (auto& b : vals[md(a)]) {
					long nxt = (a - b) / base;
					if (from.insert({nxt, b}).second) {
						nq.push_back(nxt);
					}
				}
			}
			if (from.find(0) != from.end()) {
				long u = 0;
				vector<int> ans;
				do {
					int cx = from[u];
					ans.push_back(cx);
					u = u * base + cx;
				} while (u != x);
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