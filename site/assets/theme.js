try {
  const saved = localStorage.getItem("cp-theme");
  document.documentElement.dataset.theme =
    saved ||
    (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
} catch (_) {
  document.documentElement.dataset.theme = "light";
}
