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

struct Fract {
	long num, den;

	Fract() : Fract(0) {}
	Fract(int x) : Fract(long(x)) {}
	Fract(long x) : num(x), den(1) {}
	Fract(long num, long den) : num(num), den(den) {
		norm();
	}

	void norm() {
		long cgcd = gcd(num, den);
		num /= cgcd;
		den /= cgcd;
		if (den < 0) {
			num *= -1;
			den *= -1;
		}
	}

	friend Fract& operator+=(Fract& a, const Fract& b) {
		a.num = a.num * b.den + b.num * a.den;
		a.den *= b.den;
		a.norm();
		return a;
	}

	friend Fract operator-(const Fract& f) {
		return {f.num, -f.den};
	}

	friend Fract& operator-=(Fract& a, const Fract& b) {
		return a += -b;
	}

	friend Fract& operator*=(Fract& a, const Fract& b) {
		a.num *= b.num;
		a.den *= b.den;
		a.norm();
		return a;
	}

	friend Fract& operator/=(Fract& a, const Fract& b) {
		a.num *= b.den;
		a.den *= b.num;
		a.norm();
		return a;
	}

	friend Fract operator+(Fract a, const Fract& b) {
		return a += b;
	}

	friend Fract operator-(Fract a, const Fract& b) {
		return a -= b;
	}

	friend Fract operator*(Fract a, const Fract& b) {
		return a *= b;
	}

	friend Fract operator/(Fract a, const Fract& b) {
		return a /= b;
	}

	friend bool operator<(const Fract& a, const Fract& b) {
		return (a - b).num < 0;
	}

	friend bool operator==(const Fract& a, const Fract& b) {
		return (a - b).num == 0;
	}
	
	friend ostream& operator<<(ostream& out, const Fract &f) {
		if (f.den == 1) {
			return out << f.num;
		} else if (f.den == -1) {
			return out << -f.num;
		}
		return out << f.num << " / " << f.den;
	}
};

using Point = pair<Fract, Fract>;

ostream& operator<<(ostream& out, const Point &p) {
	return out << "(" << p.first << ", " << p.second << ")";
}

struct Line {
	Fract m, b;
	bool vert;

	Line(Fract x1, Fract y1, Fract x2, Fract y2) {
		if (x1 == x2) {
			vert = true;
			m = x1;
		} else {
			vert = false;
			m = (y2 - y1) / (x2 - x1);
			b = y2 - m * x2;
		}
	}

	Fract operator()(const Fract& x) const {
		assert(!vert);
		return x * m + b;
	}

	optional<Point> inter(const Line& l) const {
		if (vert || l.vert) {
			if (vert) {
				if (l.vert) {
					return {};
				} else {
					return {{m, l(m)}};
				}
			} else {
				return {{l.m, (*this)(l.m)}};
			}
		} else if (m == l.m) {
			return {};
		}
		auto x = (l.b - b) / (m - l.m);
		return {{x, (*this)(x)}};
	}
};

const int maxn = 1005;

int n, ponl[maxn][maxn], ans, lcnt, start;
vector<int> lpoints[maxn], stk;
bool visl[maxn], visp[maxn];

void dfs(int u) {
	if (lcnt == n) {
		return;
	}
	stk.push_back(u);
	for (int i = 0; i < n; i++) {
		if (!visl[i]) {
			lcnt++;
			visl[i] = true;
			int ci = ponl[u][i];
			auto& cl = lpoints[i];
			int j;
			if (ci == -1) {
				goto end;
			}
			for (j = ci + 1; j < sz(cl); j++) {
				int v = lpoints[i][j];
				if (lcnt == n && v== start) {
					ans++;
				}
				if (visp[v]) {
					break;
				}
				visp[v] = true;
				dfs(v);
			}
			for (j--; j > ci; j--) {
				visp[lpoints[i][j]] = false;
			}
			for (j = ci - 1; j >= 0; j--) {
				int v = lpoints[i][j];
				if (lcnt == n && v== start) {
					ans++;
				}
				if (visp[v]) {
					break;
				}
				visp[v] = true;
				dfs(v);
			}
			for (j++; j < ci; j++) {
				visp[lpoints[i][j]] = false;
			}
			end:;
			lcnt--;
			visl[i] = false;
		}
	}
	stk.pop_back();
}

void solve() {
	cin >> n;
	vector<Line> lines;
	for (int i = 0; i < n; i++) {
		int x1, y1, x2, y2;
		cin >> x1 >> y1 >> x2 >> y2;
		lines.emplace_back(x1, y1, x2, y2);
	}
	map<Point, int> m;
	auto comp = [&](const Point& p) -> int {
		static int ind = 0;
		auto [it, inserted] = m.emplace(p, 0);
		if (inserted) {
			return it->second = ind++;
		}
		return it->second;
	};
	memset(ponl, -1, sizeof(ponl));
	for (int i = 0; i < n; i++) {
		vector<Point> tmp;
		for (auto& a : lines) {
			auto p = lines[i].inter(a);
			if (p) {
				tmp.push_back(p.value());
			}
		}
		sort(begin(tmp), end(tmp));
		tmp.erase(unique(begin(tmp), end(tmp)), tmp.end());
		dbg("-----------");
		dbg(i);
		for (auto& a : tmp) {
			dbg(a, comp(a));
			lpoints[i].push_back(comp(a));
		}
		for (int j = 0; j < sz(lpoints[i]); j++) {
			ponl[lpoints[i][j]][i] = j;
		}
	}
	visl[0] = true;
	lcnt = 1;
	for (int i = 0; i < sz(lpoints[0]); i++) {
		visp[lpoints[0][i]] = true;
		start = lpoints[0][i];
		stk.push_back(start);
		for (int j = i + 1; j < sz(lpoints[0]); j++) {
			int u = lpoints[0][j];
			visp[u] = true;
			dbg("INIT", u);
			dfs(u);
		}
		memset(visp, 0, sizeof(visp));
		stk.pop_back();
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}