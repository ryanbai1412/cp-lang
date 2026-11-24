// The judge's two-phase grader: `encode` sees each value in one process,
// `decode` sees only what it returned, in another, as the judge runs them.
#include "phases.h"
#include <cstdio>
#include <sys/wait.h>
#include <unistd.h>

int main() {
  int n;
  if (std::scanf("%d", &n) != 1) return 1;
  std::vector<int> a(n), s(n);
  for (int &x : a)
    if (std::scanf("%d", &x) != 1) return 1;
  std::fflush(stdout);
  int fd[2];
  if (pipe(fd)) return 1;
  pid_t first = fork();
  if (first < 0) return 1;
  if (first == 0) {
    close(fd[0]);
    for (int i = 0; i < n; ++i) s[i] = encode(a[i]);
    if (write(fd[1], s.data(), n * sizeof(int)) != (ssize_t)(n * sizeof(int))) _exit(1);
    _exit(0);
  }
  close(fd[1]);
  size_t got = 0;
  while (got < n * sizeof(int)) {
    ssize_t r = read(fd[0], (char *)s.data() + got, n * sizeof(int) - got);
    if (r <= 0) break;
    got += r;
  }
  int status;
  waitpid(first, &status, 0);
  if (got != n * sizeof(int) || !WIFEXITED(status) || WEXITSTATUS(status)) return 1;
  std::printf("%d\n", decode(s));
  return 0;
}
