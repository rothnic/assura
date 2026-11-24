"""Forward Codex sessions holding the pre-migration hook command."""

from pathlib import Path
from runpy import run_path

run_path(
    str(Path(__file__).resolve().parents[2] / ".assura/integrations/codex/assura-hook.py"),
    run_name="__main__",
)
