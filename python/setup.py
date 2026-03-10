from __future__ import annotations

from pathlib import Path

from setuptools import setup

try:
    from setuptools.command.bdist_wheel import bdist_wheel as _bdist_wheel
except ImportError:  # pragma: no cover - wheel is present in build environments
    try:
        from wheel.bdist_wheel import bdist_wheel as _bdist_wheel
    except ImportError:  # pragma: no cover - wheel is present in build environments
        _bdist_wheel = None


def _has_bundled_cli() -> bool:
    bundle_dir = Path(__file__).parent / "mnemix" / "_bin"
    return bundle_dir.is_dir() and any(bundle_dir.iterdir())


cmdclass: dict[str, type] = {}

if _bdist_wheel is not None:
    class bdist_wheel(_bdist_wheel):
        def finalize_options(self) -> None:
            super().finalize_options()
            if _has_bundled_cli():
                self.root_is_pure = False

        def get_tag(self) -> tuple[str, str, str]:
            python_tag, abi_tag, platform_tag = super().get_tag()
            if _has_bundled_cli():
                return ("py3", "none", platform_tag)
            return (python_tag, abi_tag, platform_tag)

    cmdclass["bdist_wheel"] = bdist_wheel


setup(cmdclass=cmdclass)