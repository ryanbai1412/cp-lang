#include "bits/stdc++.h"

using namespace std;

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

const int maxn = 60000, maxc = 60000 + 100, maxb = maxc * 2;

struct BIT1 {
	map<int, int> v;

	void add(int ind, int x) {
		ind++;
		while(ind <= maxb) {
			v[ind] += x;
			ind += ind & -ind;
		}
	}

	int psum(int ind) {
		int ans = 0;
		while(ind > 0) {
			ans += v[ind];
			ind -= ind & -ind;
		}
		return ans;
	}
};

struct BIT2 {
	map<int, BIT1> v;

	void add(int ind, int y, int x) {
		ind++;
		while(ind <= maxb) {
			v[ind].add(y, x);
			ind += ind & -ind;
		}
	}

	int psum(int ind, int y) {
		int ans = 0;
		while(ind > 0) {
			ans += v[ind].psum(y);
			ind -= ind & -ind;
		}
		return ans;
	}

	int sum(int x1, int y1, int x2, int y2) {
		x2++;
		y2++;
		return psum(x2, y2) - psum(x1, y2) - psum(x2, y1) + psum(x1, y1);
	}
};

struct RBIT: BIT2 {
	int get(int x, int y) {
		return psum(x + 1, y + 1);
	}

	void radd(int x1, int y1, int x2, int y2) {
		x2++;
		y2++;
		add(x1, y1, 1);
		add(x2, y1, -1);
		add(x1, y2, -1);
		add(x2, y2, 1);
	}
};

struct Query {
	int x, y, d, ind, ans;

	Query(int x, int y, int ind): x(x), y(y), d(0), ind(ind), ans(0) {}
};

void solve() {
	int n;
	cin >> n;
	vector<Query> arr;
	for(int i = 0; i < 2 * n; i++) {
		int x, y;
		cin >> x >> y;
		arr.emplace_back(x - y + maxc, x + y + maxc, i);
	}

	for(int i = 0; i < 2 * n; i += 2) {
		arr[i].d = arr[i + 1].d = max(abs(arr[i].x - arr[i + 1].x), abs(arr[i].y - arr[i + 1].y)) - 1;
	}
	sort(begin(arr), end(arr), [](const Query &lhs, const Query &rhs) {
		return lhs.d > rhs.d;
	});

	{
		BIT2 bit;
		for(auto &a: arr) {
			a.ans += bit.sum(a.x - a.d, a.y - a.d, a.x + a.d, a.y + a.d);
			bit.add(a.x, a.y, 1);
		}
	}
	reverse(begin(arr), end(arr));
	{
		RBIT bit;
		for(auto &a: arr) {
			a.ans += bit.get(a.x, a.y);
			bit.radd(a.x - a.d, a.y - a.d, a.x + a.d, a.y + a.d);
		}
	}

	sort(begin(arr), end(arr), [](const Query &lhs, const Query &rhs) {
		return lhs.ind < rhs.ind;
	});
	for(auto &a: arr) {
		cout << a.ans << endl;
	}
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
		solve();
	}
}