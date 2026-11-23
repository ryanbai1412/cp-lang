#!/usr/bin/env python3
"""Writes `updatePlugins.xml`, the custom plugin repository `cpl setup
jetbrains` registers in each IDE, so the IDE can update CP Lang and knows
where it came from. The version is the plugin's own (the workspace's).

    scripts/release/jetbrains_repo.py <out dir> <channel>
"""
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RELEASES = "https://github.com/ryanbai1412/cp-lang/releases"


def main(out_dir, channel):
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    since = next(
        line.split("=", 1)[1].strip().strip('"')
        for line in (ROOT / "editors/jetbrains/build.gradle.kts").read_text().splitlines()
        if line.strip().startswith("sinceBuild")
    )
    where = "download/edge" if channel == "edge" else "latest/download"
    out = Path(out_dir) / "updatePlugins.xml"
    out.write_text(
        "<plugins>\n"
        f'  <plugin id="dev.cplang.ide" url="{RELEASES}/{where}/cp-lang-jetbrains.zip" version="{version}">\n'
        f'    <idea-version since-build="{since}"/>\n'
        "  </plugin>\n"
        "</plugins>\n"
    )
    print(out)


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__.strip().splitlines()[-1].strip())
    main(sys.argv[1], sys.argv[2])
