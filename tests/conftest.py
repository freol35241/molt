"""Pytest configuration and fixtures for molt integration tests."""

import os
import shutil
import subprocess
import sys
import venv
from pathlib import Path

import pytest

# Project root directory
ROOT_DIR = Path(__file__).parent.parent
EXAMPLES_DIR = ROOT_DIR / "examples"
MINIMAL_DIR = EXAMPLES_DIR / "minimal"
PRIMITIVES_DIR = EXAMPLES_DIR / "primitives"
COLLECTIONS_DIR = EXAMPLES_DIR / "collections"
MULTI_MODEL_DIR = EXAMPLES_DIR / "multi-model"


def get_env_with_cargo():
    """Get environment with cargo bin directory in PATH."""
    env = os.environ.copy()
    # Ensure ~/.cargo/bin is in PATH for cargo-component
    home = Path.home()
    cargo_bin = home / ".cargo" / "bin"
    if cargo_bin.exists():
        current_path = env.get("PATH", "")
        if str(cargo_bin) not in current_path:
            env["PATH"] = f"{cargo_bin}:{current_path}"
    return env


@pytest.fixture(scope="session")
def molt_binary() -> Path:
    """Build molt in release mode and return path to binary."""
    # Build molt
    result = subprocess.run(
        ["cargo", "build", "--release"],
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        pytest.fail(f"Failed to build molt:\n{result.stderr}")

    binary = ROOT_DIR / "target" / "release" / "molt"
    if not binary.exists():
        pytest.fail(f"molt binary not found at {binary}")

    return binary


@pytest.fixture(scope="session")
def cargo_component_available() -> bool:
    """Check if cargo-component is available."""
    result = subprocess.run(
        ["cargo", "component", "--version"],
        capture_output=True,
        text=True,
        env=get_env_with_cargo(),
    )
    return result.returncode == 0


@pytest.fixture(scope="session")
def built_minimal_example(molt_binary: Path, cargo_component_available: bool, tmp_path_factory) -> Path:
    """Build the minimal example and return path to generated Python package."""
    if not cargo_component_available:
        pytest.skip("cargo-component not available")

    # Create a temporary directory for the build
    build_dir = tmp_path_factory.mktemp("minimal-build")

    # Copy the example to temp dir (so we don't pollute the source tree)
    example_copy = build_dir / "minimal"
    shutil.copytree(MINIMAL_DIR, example_copy)

    # Step 1: Compile WASM using cargo component directly
    # (This avoids issues with nested subprocess spawning)
    result = subprocess.run(
        ["cargo", "component", "build", "--release"],
        cwd=example_copy,
        capture_output=True,
        text=True,
        timeout=300,  # 5 minute timeout for WASM compilation
        env=get_env_with_cargo(),
    )
    if result.returncode != 0:
        pytest.fail(f"cargo component build failed:\nstdout: {result.stdout}\nstderr: {result.stderr}")

    # Step 2: Run molt build with --skip-compile to generate Python package
    result = subprocess.run(
        [str(molt_binary), "build", "--skip-compile"],
        cwd=example_copy,
        capture_output=True,
        text=True,
        timeout=60,
        env=get_env_with_cargo(),
    )
    if result.returncode != 0:
        pytest.fail(f"molt build failed:\nstdout: {result.stdout}\nstderr: {result.stderr}")

    # Return path to generated Python package
    python_pkg_dir = example_copy / "dist" / "python"
    if not python_pkg_dir.exists():
        pytest.fail(f"Generated Python package not found at {python_pkg_dir}")

    return python_pkg_dir


@pytest.fixture(scope="session")
def installed_package_venv(built_minimal_example: Path, tmp_path_factory) -> Path:
    """
    Create a fresh virtual environment with the generated package installed.

    Returns the path to the venv directory.
    """
    venv_dir = tmp_path_factory.mktemp("test-venv")

    # Create virtual environment
    venv.create(venv_dir, with_pip=True)

    # Determine pip path based on platform
    if sys.platform == "win32":
        pip_path = venv_dir / "Scripts" / "pip"
    else:
        pip_path = venv_dir / "bin" / "pip"

    # Install wasmtime first (required dependency)
    result = subprocess.run(
        [str(pip_path), "install", "wasmtime>=21.0.0"],
        capture_output=True,
        text=True,
        timeout=120,
    )
    if result.returncode != 0:
        pytest.fail(f"Failed to install wasmtime:\n{result.stderr}")

    # Install the generated package
    result = subprocess.run(
        [str(pip_path), "install", str(built_minimal_example)],
        capture_output=True,
        text=True,
        timeout=120,
    )
    if result.returncode != 0:
        pytest.fail(f"Failed to install generated package:\n{result.stderr}")

    return venv_dir


@pytest.fixture
def python_in_venv(installed_package_venv: Path) -> Path:
    """Return path to Python executable in the test venv."""
    if sys.platform == "win32":
        return installed_package_venv / "Scripts" / "python"
    else:
        return installed_package_venv / "bin" / "python"
