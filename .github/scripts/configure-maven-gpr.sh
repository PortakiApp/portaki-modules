#!/usr/bin/env bash
set -euo pipefail

mkdir -p ~/.m2
python3 <<'PY'
import os
import pathlib

workspace = pathlib.Path(os.environ["GITHUB_WORKSPACE"])
root = workspace / ".github/maven-gpr-settings.xml"
text = root.read_text()
text = text.replace("__GPR_ACTOR__", os.environ["GPR_ACTOR"])
text = text.replace("__GPR_TOKEN__", os.environ["GPR_TOKEN"])
pathlib.Path.home().joinpath(".m2/settings.xml").write_text(text)
PY
