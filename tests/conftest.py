import sys
from pathlib import Path

# Ensure sutra_core is importable when running tests from repo root
ROOT = Path(__file__).resolve().parents[1]
CORE = ROOT / "packages" / "sutra-core"
if str(CORE) not in sys.path:
    sys.path.insert(0, str(CORE))


def pytest_configure(config):
    # Register custom markers
    config.addinivalue_line("markers", "integration: marks tests as integration (deselect with -m 'not integration')")
