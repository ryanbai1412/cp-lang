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
	typedef double ll;
	typedef pair<int, int> pii;
	typedef vector<int> vi;

	struct Line {
		mutable ll k, m, p;

		bool operator<(const Line &o) const {return k < o.k;}

		bool operator<(ll x) const {return p < x;}
	};

	struct LineContainer: multiset<Line, less<>> {
		// (for doubles, use inf = 1/.0, div(a,b) = a/b)
		static constexpr ll inf = DBL_MAX;

		ll div(ll a, ll b) { // floored division
			return a / b;
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
			return l.k * x + l.m;
		}
	};

#undef rep
#undef all
}
//endregion

void solve() {
	int n, m;
	cin >> n >> m;
	double arr[n][3];
	for(int i = 0; i < n; i++) {
		for(int j = 0; j < 3; j++) {
			cin >> arr[i][j];
		}
	}
	LineContainer cht;
	double dp[n][2] {};
	double psum = 1;
	// 0: have stocks
	// 1: have money
	for(int i = n - 1; i >= 0; i--) {
		if(sz(cht)) {
			dp[i][0] = cht.query(arr[i][2]);
		}
		psum = max(psum, 1 / (arr[i][2] * arr[i][0] + 1 * arr[i][1]) * dp[i][0]);
		dp[i][1] = psum;
		cht.add(arr[i][0] * dp[i][1], arr[i][1] * dp[i][1]);
	}
	cout << dp[0][1] * m << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	cout << fixed << setprecision(3);
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}