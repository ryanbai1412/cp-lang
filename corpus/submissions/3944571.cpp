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

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

struct Event {
	int t, ind, points;

	bool operator<(const Event& e) const {
		return tie(t, points) < tie(e.t, e.points);
	}
};

void solve() {
	int _, n, m;
	cin >> _ >> n >> m;
	vector<Event> events;
	auto convp = [&](int t) -> int {
		return t ? 3 : 1;
	};
	for (int i = 0; i < n; i++) {
		int l, r, t;
		cin >> l >> r >> t;
		t = convp(t);
		events.push_back({l, 0, t});
		events.push_back({r + 1, 0, -t});
	}
	for (int i = 0; i < m; i++) {
		int l, r, t;
		cin >> l >> r >> t;
		t = convp(t);
		events.push_back({l, 1, t});
		events.push_back({r + 1, 1, -t});
	}
	sort(begin(events), end(events));
	int ans = 0, lt = 0;
	int points[2] {};
	for (int i = 0; i < sz(events); ) {
		int ct = events[i].t;
		ans += (ct - lt) * *max_element(begin(points), end(points));
		dbg(points[0], points[1], lt, ct, ans);
		for (; i < sz(events) && events[i].t == ct; i++) {
			auto [_, ind, p] = events[i];
			points[ind] += p;
		}
		lt = ct;
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