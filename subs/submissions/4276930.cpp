#pragma GCC optimize("Ofast,unroll-loops")

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

namespace IO {
const int BUFFER_SIZE = 1 << 16;

char input_buffer[BUFFER_SIZE];
size_t input_pos = 0, input_len = 0;

char output_buffer[BUFFER_SIZE];
int output_pos = 0;

char number_buffer[100];
uint8_t lookup[100];

void _update_input_buffer() {
	input_len = fread(input_buffer, sizeof(char), BUFFER_SIZE, stdin);
	input_pos = 0;

	if (input_len == 0)
		input_buffer[0] = EOF;
}

inline char next_char(bool advance = true) {
	if (input_pos >= input_len)
		_update_input_buffer();

	return input_buffer[advance ? input_pos++ : input_pos];
}

template <typename T>
inline void read_int(T& number) {
	bool negative = false;
	number = 0;

	while (!isdigit(next_char(false)))
		if (next_char() == '-')
			negative = true;

	do {
		number = 10 * number + (next_char() - '0');
	} while (isdigit(next_char(false)));

	if (negative)
		number = -number;
}

template <typename T, typename... Args>
inline void read_int(T& number, Args&... args) {
	read_int(number);
	read_int(args...);
}

void _flush_output() {
	fwrite(output_buffer, sizeof(char), output_pos, stdout);
	output_pos = 0;
}

inline void write_char(char c) {
	if (output_pos == BUFFER_SIZE)
		_flush_output();

	output_buffer[output_pos++] = c;
}

template <typename T>
inline void write_int(T number, char after = '\0') {
	if (number < 0) {
		write_char('-');
		number = -number;
	}

	int length = 0;

	while (number >= 10) {
		uint8_t lookup_value = lookup[number % 100];
		number /= 100;
		number_buffer[length++] = char((lookup_value & 15) + '0');
		number_buffer[length++] = char((lookup_value >> 4) + '0');
	}

	if (number != 0 || length == 0)
		write_char(char(number + '0'));

	for (int i = length - 1; i >= 0; i--)
		write_char(number_buffer[i]);

	if (after)
		write_char(after);
}

void init() {
	// Make sure _flush_output() is called at the end of the program.
	bool exit_success = atexit(_flush_output) == 0;
	assert(exit_success);

	for (int i = 0; i < 100; i++)
		lookup[i] = uint8_t((i / 10 << 4) + i % 10);
}
}  // namespace IO

const int maxn = 5e5 + 5, bsize = 900, bcnt = maxn / bsize + 1;

using u64 = uint64_t;

u64 mask(int x) {
	return (u64(1) << x) - 1;
}

struct ST {
	u64 v[1 << 20] {}, tmp[1 << 20];

	void insert(int x) {
		x += 1 << 24;
		if (!v[x >> 6]) {
			v[x >> 6] |= u64(1) << (x & 63);
			x >>= 6;
			if (!v[x >> 6]) {
				v[x >> 6] |= u64(1) << (x & 63);
				x >>= 6;
				if (!v[x >> 6]) {
					v[x >> 6] |= u64(1) << (x & 63);
					x >>= 6;
					if (!v[x >> 6]) {
						v[x >> 6] |= u64(1) << (x & 63);
						x >>= 6;
			
					} else {
						v[x >> 6] |= u64(1) << (x & 63);
					}
				} else {
					v[x >> 6] |= u64(1) << (x & 63);
				}
			} else {
				v[x >> 6] |= u64(1) << (x & 63);
			}
		} else {
			v[x >> 6] |= u64(1) << (x & 63);
		}
	}

	void remove(int x) {
		x += 1 << 24;
		v[x >> 6] ^= u64(1) << (x & 63);
		x >>= 6;
		if (!v[x]) {
			v[x >> 6] ^= u64(1) << (x & 63);
			x >>= 6;
			if (!v[x]) {
				v[x >> 6] ^= u64(1) << (x & 63);
				x >>= 6;
				if (!v[x]) {
					v[x >> 6] ^= u64(1) << (x & 63);
					x >>= 6;
					if (!v[x]) {
			
					}
				}
			}
		}
	}

	int query(int x) {
		x += 1 << 24;
#pragma GCC unroll 4
		for (int i = 0; i < 4; i++) {
			u64 cmsk = mask(x & 63);
			x >>= 6;
			if (v[x] & cmsk) {
				x = (x << 6) | (63 - __builtin_clzll(v[x] & cmsk));
#pragma GCC unroll 3
				for (int j = 0; j < i; j++) {
					int bit = 63 - __builtin_clzll(v[x]);
					x = (x << 6) | bit;
				}
				return x - (1 << 24);
			}
		}
		return -1;
	}
};

struct BDS {
	set<pair<int, int>> x;

	pair<int, int> insert(int a, int b) {
		auto [it, _] = x.emplace(a, b);
		if (sz(x) == 1) {
			return {-1, -1};
		} else if (it == x.begin()) {
			return {-1, (++it)->second};
		} else if (it == --x.end()) {
			return {(--it)->second, -1};
		} else {
			return {prev(it)->second, (++it)->second};
		}
	}

	pair<int, int> erase(int a, int b) {
		auto it = x.erase(x.find({a, b}));
		if (!sz(x)) {
			return {-1, -1};
		} else if (it == x.begin()) {
			return {-1, it->second};
		} else if (it == x.end()) {
			return {(--it)->second, -1};
		} else {
			int tmp = it->second;
			return {(--it)->second, tmp};
		}
	}
};

struct DS {
	ST sta {}, stb {};
	int arr[maxn] {};

	int get(int x) const {
		if (0 <= x && x < maxn) {
			return arr[x];
		}
		return -1;
	}

	pair<int, int> answer(int a) {
		return {get(sta.query(a)), get(maxn - stb.query(maxn - a))};
	}

	pair<int, int> insert(int a, int b) {
		arr[a] = b;
		sta.insert(a);
		stb.insert(maxn - a);
		return answer(a);
	}

	pair<int, int> erase(int a, int) {
		sta.remove(a);
		stb.remove(maxn - a);
		return answer(a);
	}
};

struct Query {
	int l, r, qind, lind;

	Query(int l, int r, int qind) : l(l), r(r), qind(qind), lind(l / bsize) {}

	bool operator<(const Query& e) const {
		if (lind == e.lind) {
			if (r == e.r) {
				return l < e.l;
			}
			if (lind & 1) {
				return r > e.r;
			} else {
				return r < e.r;
			}
		}
		return lind < e.lind;
	}
};

long cans = 0;

template <typename T, typename U>
void solve(vector<Query> queries, const T& add, const U& rm) {
	sort(begin(queries), end(queries));
	long ans[sz(queries)];
	int cl = 0, cr = 0;
	for (auto& [l, r, ind, _] : queries) {
		if (min(cr, r) <= max(cl, l)) {
			for (int i = cl; i < cr; i++) {
				rm(i);
			}
			for (int i = l; i < r; i++) {
				add(i);
			}
		} else {
			for (int i = cl; i < l; i++) {
				rm(i);
			}
			for (int i = cl - 1; i >= l; i--) {
				add(i);
			}
			for (int i = cr; i < r; i++) {
				add(i);
			}
			for (int i = cr - 1; i >= r; i--) {
				rm(i);
			}
		}
		ans[ind] = cans;
		tie(cl, cr) = tie(l, r);
	}
	for (auto& a : ans) {
		IO::write_int(a, '\n');
	}
}

void solve() {
	int n, q;
	IO::read_int(n, q);
	int arr[n];
	for (auto& a : arr) {
		IO::read_int(a);
	}
	vector<Query> queries;
	for (int i = 0; i < q; i++) {
		int l, r;
		IO::read_int(l, r);
		l--;
		queries.emplace_back(l, r, i);
	}
	DS ds;
	auto add = [&](int i) -> void {
		auto [a, b] = ds.insert(arr[i], i);
		dbg("ADD", arr[i], i, a, b);
		if (a != -1 && b != -1) {
			cans -= abs(a - b);
			cans += abs(i - a) + abs(b - i);
		} else {
			if (a != -1) {
				cans += abs(a - i);
			}
			if (b != -1) {
				cans += abs(b - i);
			}
		}
	};
	auto rm = [&](int i) -> void {
		auto [a, b] = ds.erase(arr[i], i);
		dbg("RM", arr[i], i, a, b);
		if (a != -1 && b != -1) {
			cans += abs(a - b);
			cans -= abs(i - a) + abs(b - i);
		} else {
			if (a != -1) {
				cans -= abs(a - i);
			}
			if (b != -1) {
				cans -= abs(b - i);
			}
		}
	};
	solve(queries, add, rm);
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	IO::init();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}