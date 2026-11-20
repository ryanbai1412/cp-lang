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
#define dbg(...) 1412
#endif

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxm = 1005;

struct Event {
	ld t;
	int add;

	bool operator<(const Event& e) const {
		return pair<int, int>{t, -add} < pair<int, int>{e.t, -e.add};
	}
};

struct DS {
	vector<Event> events;

	void add(ld l, ld r, int x) {
		events.push_back({l, x});
		events.push_back({r, -x});
	}

	int compute() {
		sort(begin(events), end(events));
		int ans = 0, cur = 0;
		for (auto& [_, x] : events) {
			ans = max(ans, cur += x);
		}
		return ans;
	}
} arr[maxm];

void solve() {
	int n, r;
	cin >> n >> r;
	ld rr = r * r;
	ld change[maxm];
	for (int i = 0; i < maxm; i++) {
		ld x = rr - i * i;
		if (x < 0) {
			change[i] = -1;
		} else {
			change[i] = sqrt(x);
		}
	}
	auto upd = [&](int x, ld l, ld r, int p) -> void {
		if (0 <= x && x < maxm) {
			dbg(x, l, r, p);
			arr[x].add(l, r, p);
		}
	};
	for (int i = 0; i < n; i++) {
		int x, y, t;
		cin >> x >> y >> t;
		t = t ? 3 : 1;
		for (int j = 0; j < maxm && change[j] >= 0; j++) {
			upd(x - j, y - change[j], y + change[j], t);
			if (j) {
				upd(x + j, y - change[j], y + change[j], t);
			}
		}
	}
	int ans = 0;
	for (int i = 0; i < maxm; i++) {
		ans = max(ans, arr[i].compute());
	}
	cout << ans << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}