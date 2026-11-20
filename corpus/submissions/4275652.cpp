#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("avx2")

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

const int maxn = 2e5 + 5, bsize = 400, bcnt = maxn / bsize + 1;

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
	auto check = [&]() -> bool {
		for (int i = 0; i < n; i++) {
			if (abs(arr[i] - (i + 1)) > 10) {
				return false;
			}
		}
		return true;
	};
	if (check()) {
		const int bsz = 3e6;
		array<int, 2> buf[bsz * 3];
		int l = bsz, r = bsz;
		auto fix = [&]() -> void {
			copy(buf + l, buf + r, buf + bsz);
			int csz = r - l;
			l = bsz;
			r = l + csz;
		};
		auto lb = [&](int i) -> pair<int, bool> {
			if (l == r) {
				return {l, true};
			}
			if (abs(arr[i] - buf[l][0]) <= 21) {
				int it = l;
				buf[r][0] = 1e9;
#pragma GCC unroll 30
				while (buf[it++][0] < arr[i])
					;
				it--;
				return {it, true};
			} else {
				int it = r;
				buf[l - 1][0] = -1e9;
#pragma GCC unroll 30
				while (it != l && arr[i] <= buf[--it][0])
					;
				++it;
				return {it, false};
			}
		};
		auto add = [&](int i) -> void {
			if (l == r) {
				buf[l][0] = arr[i];
				buf[l][1] = i;
				r++;
				return;
			}
			auto [it, left] = lb(i);
			dbg("ADD", l, it, r);
			if (left) {
				if (!l) {
					fix();
				}
				copy(buf + l, buf + it, buf + l - 1);
				l--;
				it--;
			} else {
				if (r == bsz * 3) {
					fix();
				}
				copy(buf + it, buf + r, buf + it + 1);
				r++;
			}
			dbg("ADD", l, it, r);
			buf[it][0] = arr[i];
			buf[it][1] = i;
			if (it == l) {
				cans += abs(buf[it + 1][1] - i);
			} else if (it == r - 1) {
				cans += abs(buf[it - 1][1] - i);
			} else {
				int a = buf[it - 1][1], b = buf[it + 1][1];
				cans -= abs(a - b);
				cans += abs(i - a) + abs(b - i);
			}
		};
		auto rm = [&](int i) -> void {
			auto [it, left] = lb(i);
			dbg("RM", l, it, r);
			if (left) {
				copy(buf + l, buf + it, buf + l + 1);
				l++;
				it++;
			} else {
				copy(buf + it + 1, buf + r, buf + it);
				r--;
			}
			dbg("RM", l, it, r);
			if (l == r) {
				return;
			}
			if (it == l) {
				cans -= abs(buf[it][1] - i);
			} else if (it == r) {
				cans -= abs(buf[it - 1][1] - i);
			} else {
				int a = buf[it][1], b = buf[it - 1][1];
				cans -= abs(i - a) + abs(i - b);
				cans += abs(a - b);
			}
		};
		solve(queries, add, rm);
	} else {
		set<pair<int, int>> vals;
		auto add = [&](int i) -> void {
			auto [it, _] = vals.emplace(arr[i], i);
			if (it == vals.begin()) {
				if (sz(vals) >= 2) {
					cans += abs((++it)->second - i);
				}
			} else if (it == (--vals.end())) {
				if (sz(vals) >= 2) {
					cans += abs((--it)->second - i);
				}
			} else {
				int a = prev(it)->second, b = (++it)->second;
				cans -= abs(a - b);
				cans += abs(i - a) + abs(b - i);
			}
		};
		auto rm = [&](int i) -> void {
			auto it = vals.erase(vals.find({arr[i], i}));
			if (!sz(vals)) {
				return;
			}
			if (it == vals.begin()) {
				cans -= abs(it->second - i);
			} else if (it == vals.end()) {
				cans -= abs((--it)->second - i);
			} else {
				int a = it->second, b = (--it)->second;
				cans -= abs(i - a) + abs(i - b);
				cans += abs(a - b);
			}
		};
		solve(queries, add, rm);
	}
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