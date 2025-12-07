"""Tests for molt CLI commands."""

import subprocess
from pathlib import Path

import pytest

from conftest import MINIMAL_DIR, PRIMITIVES_DIR, ROOT_DIR, get_env_with_cargo


class TestMoltCheck:
    """Tests for `molt check` command."""

    def test_check_minimal_example(self, molt_binary: Path):
        """molt check should validate the minimal example successfully."""
        result = subprocess.run(
            [str(molt_binary), "check"],
            cwd=MINIMAL_DIR,
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"molt check failed:\n{result.stderr}"
        assert "All checks passed!" in result.stdout
        assert "params:" in result.stdout
        assert "inputs:" in result.stdout
        assert "outputs:" in result.stdout

    def test_check_with_manifest_path(self, molt_binary: Path):
        """molt check should accept --manifest flag."""
        manifest_path = MINIMAL_DIR / "molt.toml"
        result = subprocess.run(
            [str(molt_binary), "check", "--manifest", str(manifest_path)],
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"molt check failed:\n{result.stderr}"
        assert "All checks passed!" in result.stdout

    def test_check_primitives_example(self, molt_binary: Path):
        """molt check should validate the primitives example with various types."""
        result = subprocess.run(
            [str(molt_binary), "check"],
            cwd=PRIMITIVES_DIR,
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"molt check failed:\n{result.stderr}"
        assert "All checks passed!" in result.stdout
        # Check that multiple types are recognized
        assert "scale_f64" in result.stdout
        assert "enabled" in result.stdout

    def test_check_missing_manifest(self, molt_binary: Path, tmp_path: Path):
        """molt check should fail gracefully with missing manifest."""
        result = subprocess.run(
            [str(molt_binary), "check"],
            cwd=tmp_path,
            capture_output=True,
            text=True,
        )

        assert result.returncode != 0
        assert "Failed to load manifest" in result.stderr or "molt.toml" in result.stderr


class TestMoltBuild:
    """Tests for `molt build` command."""

    def test_build_creates_python_package(
        self, molt_binary: Path, cargo_component_available: bool, tmp_path: Path
    ):
        """molt build should generate a Python package."""
        if not cargo_component_available:
            pytest.skip("cargo-component not available")

        # Copy example to temp dir
        import shutil

        example_copy = tmp_path / "minimal"
        shutil.copytree(MINIMAL_DIR, example_copy)

        # Step 1: Compile WASM using cargo component directly
        result = subprocess.run(
            ["cargo", "component", "build", "--release"],
            cwd=example_copy,
            capture_output=True,
            text=True,
            timeout=300,
            env=get_env_with_cargo(),
        )
        assert result.returncode == 0, f"cargo component build failed:\n{result.stdout}\n{result.stderr}"

        # Step 2: Run molt build with --skip-compile
        result = subprocess.run(
            [str(molt_binary), "build", "--skip-compile"],
            cwd=example_copy,
            capture_output=True,
            text=True,
            timeout=60,
            env=get_env_with_cargo(),
        )

        assert result.returncode == 0, f"molt build failed:\n{result.stdout}\n{result.stderr}"
        assert "Build complete!" in result.stdout

        # Check generated files exist
        python_pkg = example_copy / "dist" / "python"
        assert python_pkg.exists(), "dist/python directory not created"

        pkg_dir = python_pkg / "minimal_example"
        assert pkg_dir.exists(), "minimal_example package directory not created"

        expected_files = [
            "pyproject.toml",
            "minimal_example/__init__.py",
            "minimal_example/_runtime.py",
            "minimal_example/_wasm.py",
            "minimal_example/scale.py",
            "minimal_example/py.typed",
        ]

        for file_path in expected_files:
            full_path = python_pkg / file_path
            assert full_path.exists(), f"Expected file not found: {file_path}"

    def test_build_with_target_flag(
        self, molt_binary: Path, cargo_component_available: bool, tmp_path: Path
    ):
        """molt build --target python should only build Python target."""
        if not cargo_component_available:
            pytest.skip("cargo-component not available")

        import shutil

        example_copy = tmp_path / "minimal"
        shutil.copytree(MINIMAL_DIR, example_copy)

        # Step 1: Compile WASM using cargo component directly
        result = subprocess.run(
            ["cargo", "component", "build", "--release"],
            cwd=example_copy,
            capture_output=True,
            text=True,
            timeout=300,
            env=get_env_with_cargo(),
        )
        assert result.returncode == 0, f"cargo component build failed:\n{result.stderr}"

        # Step 2: Run molt build with --skip-compile and --target
        result = subprocess.run(
            [str(molt_binary), "build", "--skip-compile", "--target", "python"],
            cwd=example_copy,
            capture_output=True,
            text=True,
            timeout=60,
            env=get_env_with_cargo(),
        )

        assert result.returncode == 0, f"molt build failed:\n{result.stderr}"
        assert "Generating python package" in result.stdout


class TestMoltInit:
    """Tests for `molt init` command."""

    def test_init_creates_project(self, molt_binary: Path, tmp_path: Path):
        """molt init should create a new project structure."""
        result = subprocess.run(
            [str(molt_binary), "init", "my-test-project"],
            cwd=tmp_path,
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"molt init failed:\n{result.stderr}"

        project_dir = tmp_path / "my-test-project"
        assert project_dir.exists()

        expected_files = [
            "molt.toml",
            "Cargo.toml",
            "src/lib.rs",
            "wit/example.wit",
        ]

        for file_path in expected_files:
            full_path = project_dir / file_path
            assert full_path.exists(), f"Expected file not found: {file_path}"

    def test_init_fails_if_exists(self, molt_binary: Path, tmp_path: Path):
        """molt init should fail if directory already exists."""
        # Create directory first
        existing_dir = tmp_path / "existing-project"
        existing_dir.mkdir()

        result = subprocess.run(
            [str(molt_binary), "init", "existing-project"],
            cwd=tmp_path,
            capture_output=True,
            text=True,
        )

        assert result.returncode != 0
        assert "already exists" in result.stderr


class TestMoltHelp:
    """Tests for molt help output."""

    def test_help_flag(self, molt_binary: Path):
        """molt --help should show usage information."""
        result = subprocess.run(
            [str(molt_binary), "--help"],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0
        assert "Model Once, Load Trivially" in result.stdout
        assert "build" in result.stdout
        assert "check" in result.stdout
        assert "init" in result.stdout

    def test_version_flag(self, molt_binary: Path):
        """molt --version should show version."""
        result = subprocess.run(
            [str(molt_binary), "--version"],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0
        assert "molt" in result.stdout
