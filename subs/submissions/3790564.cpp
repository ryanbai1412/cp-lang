#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

//region kactl
namespace {
#define rep(i, a, b) for(int i = a; i < (b); ++i)
#define all(x) begin(x), end(x)
#define ll long
	typedef pair<int, int> pii;
	typedef vector<int> vi;

	struct Line {
		mutable ll k, m, p;

		bool operator<(const Line &o) const {return k < o.k;}

		bool operator<(ll x) const {return p < x;}
	};

	struct LineContainer: multiset<Line, less<>> {
		// (for doubles, use inf = 1/.0, div(a,b) = a/b)
		static const ll inf = LLONG_MAX;

		ll div(ll a, ll b) { // floored division
			return a / b - ((a ^ b) < 0 && a % b);
		}

		bool isect(iterator x, iterator y) {
			if(y == end())
				return x->p = inf, 0;
			if(x->k == y->k)
				x->p = x->m > y->m ? inf : -inf;
			else
				x->p = div(y->m - x->m, x->k - y->k);
			return x->p >= y->p;
		}

		void add(ll k, ll m) {
			k = -k;
			m = -m;
			auto z = insert({k, m, 0}), y = z++, x = y;
			while(isect(y, z))
				z = erase(z);
			if(x != begin() && isect(--x, y))
				isect(x, y = erase(y));
			while((y = x) != begin() && (--x)->p >= y->p)
				isect(x, erase(y));
		}

		ll query(ll x) {
			assert(!empty());
			auto l = *lower_bound(x);
			return -(l.k * x + l.m);
		}
	};

#undef rep
#undef all
}
//endregion

using CHT = LineContainer;

enum EventType {
	ADD,
	COMPUTE
};

struct Event {
	EventType type;
	int time, data;

	bool operator<(const Event &e) {
		return tie(time, type) < tie(e.time, e.type);
	}
};

void solve() {
	int n, m, ax, bx, cx;
	cin >> n >> m >> ax >> bx >> cx;

	int edges[m + 1][4];
	for(int i = 0; i < m; i++) {
		for(int j = 0; j < 4; j++) {
			cin >> edges[i][j];
		}
		edges[i][0]--;
		edges[i][1]--;
	}
	for(int i = 0; i < 4; i++) {
		edges[m][i] = 0;
	}

	m++;
	long dp[m];
	CHT cht[n];
	for(int i = 0; i < n; i++) {
		cht[i].add(0, 4e18);
	}

	vector<Event> events;
	for(int i = 0; i < m; i++) {
		auto &[u, v, s, e] = edges[i];
		events.push_back({EventType::ADD, -s, i});
		events.push_back({EventType::COMPUTE, -e, i});
	}
	sort(begin(events), end(events));
	for(auto &[type, _, ind]: events) {
		auto &[u, v, s, e] = edges[ind];
		if(type == EventType::ADD) {
			cht[u].add(-2 * ax * s, ax * s * s + bx * s + cx + dp[ind]);
		}else {
			if(v == n - 1) {
				dp[ind] = e;
			}else {
				dp[ind] = cht[v].query(e) + ax * e * e - bx * e;
			}
			dbg(ind, dp[ind]);
		}
	}
	cout << dp[m - 1] << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}