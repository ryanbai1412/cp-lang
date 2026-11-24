// The interactor for ask_loop: answers "? i" with i + 1 and prints the
// final "! s" line to stderr, which run.py compares.
#include <cstdio>
#include <cstring>
int main() {
    char op[4];
    long long v;
    while (scanf("%3s %lld", op, &v) == 2) {
        if (op[0] == '!') { fprintf(stderr, "%lld\n", v); return 0; }
        printf("%lld\n", v + 1);
        fflush(stdout);
    }
    return 1;
}
