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

const int maxn = 3e5, bsize = 2000, bcnt = maxn / bsize + 1;

struct Block {
	int max_l = 0, suf_min = 1e9, block_min = 1e9, local_min[bsize] {}, each_end[bsize] {},
		each_len[bsize] {};

	Block() {
		fill(begin(local_min), end(local_min), 1e9);
		iota(begin(each_end), end(each_end), 0);
	}

	int query_min(int ind) { return min(suf_min, local_min[ind]); }

	void insert(int l, int r) {
		block_min = min(block_min, r);
		if (r < local_min[l]) {
			local_min[l] = r;
			max_l = max(max_l, l);
		}
	}
	
	void compute() {
		for (int i = min(bsize - 2, max_l); i >= 0; i--) {
			local_min[i] = min(local_min[i], local_min[i + 1]);
			if (local_min[i] < bsize) {
				each_end[i] = each_end[local_min[i]];
				each_len[i] = each_len[local_min[i]] + 1;
			}
		}
		max_l = 0;
	}
} blocks[bcnt];

void solve() {
	int n, q;
	IO::read_int(q, n);
	bool lastq = true;
	vector<int> upds;
	int mcblock = 0;
	while (q--) {
		int t, l, r;
		IO::read_int(t, l, r);
		l--;
		if (t == 1) {
			lastq = false;
			int cblock = l / bsize, cadd = cblock * bsize;
			blocks[cblock].insert(l - cadd, r - cadd);
			upds.push_back(cblock);
			mcblock = max(mcblock, cblock);
		} else {
			if (!lastq) {
				for (auto& a : upds) {
					blocks[a].compute();
				}
				for (int i = mcblock - 1; i >= 0; i--) {
					blocks[i].suf_min =
						min({blocks[i].suf_min, blocks[i + 1].block_min + bsize,
							 blocks[i + 1].suf_min + bsize});
				}
				upds.clear();
				mcblock = 0;
			}
			lastq = true;
			int ans = 0;
			int bend = r / bsize;
			while (l / bsize < bend) {
				int cblock = l / bsize, cl = l % bsize, cadd = cblock * bsize;
				ans += blocks[cblock].each_len[cl];
				l = blocks[cblock].each_end[cl];
				l = cadd + blocks[cblock].query_min(l);
				if (l <= r) {
					ans++;
				}
			}
			int cblock = bend, cadd = cblock * bsize;
			while (l < r) {
				l = cadd + blocks[cblock].local_min[l - cadd];
				if (l <= r) {
					ans++;
				}
			}
			IO::write_int(ans, '\n');
		}
	}
}

int main() {
	IO::init();
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