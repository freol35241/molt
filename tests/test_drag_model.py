"""Tests for the generated drag model Python package.

These tests verify that the generated Python package:
1. Can be imported successfully
2. Has the expected API (create_drag, dataclasses, etc.)
3. Produces correct numerical outputs
"""

import subprocess
import textwrap
from pathlib import Path

import pytest


class TestPackageImport:
    """Tests that the generated package can be imported."""

    def test_import_package(self, python_in_venv: Path):
        """The physics_models package should be importable."""
        code = "import physics_models; print('OK')"
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Import failed:\n{result.stderr}"
        assert "OK" in result.stdout

    def test_import_create_drag(self, python_in_venv: Path):
        """create_drag should be importable from physics_models."""
        code = "from physics_models import create_drag; print('OK')"
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Import failed:\n{result.stderr}"
        assert "OK" in result.stdout

    def test_import_dataclasses(self, python_in_venv: Path):
        """DragParams, DragInputs, DragOutputs should be importable."""
        code = textwrap.dedent("""
            from physics_models import DragParams, DragInputs, DragOutputs
            print('OK')
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Import failed:\n{result.stderr}"
        assert "OK" in result.stdout


class TestDragModelAPI:
    """Tests for the drag model API."""

    def test_create_drag_returns_callable(self, python_in_venv: Path):
        """create_drag should return a callable model."""
        code = textwrap.dedent("""
            from physics_models import create_drag
            model = create_drag(area=10.0, cd=0.3)
            print(f"callable: {callable(model)}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "callable: True" in result.stdout

    def test_model_has_params_attribute(self, python_in_venv: Path):
        """The model should have a params attribute with the input parameters."""
        code = textwrap.dedent("""
            from physics_models import create_drag
            model = create_drag(area=10.0, cd=0.3)
            print(f"area: {model.params.area}")
            print(f"cd: {model.params.cd}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "area: 10.0" in result.stdout
        assert "cd: 0.3" in result.stdout

    def test_model_returns_outputs(self, python_in_venv: Path):
        """Calling the model should return a DragOutputs object."""
        code = textwrap.dedent("""
            from physics_models import create_drag, DragOutputs
            model = create_drag(area=10.0, cd=0.3)
            result = model(rho=1.225, velocity=20.0)
            print(f"is_outputs: {isinstance(result, DragOutputs)}")
            print(f"has_drag_force: {hasattr(result, 'drag_force')}")
            print(f"has_dynamic_pressure: {hasattr(result, 'dynamic_pressure')}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "is_outputs: True" in result.stdout
        assert "has_drag_force: True" in result.stdout
        assert "has_dynamic_pressure: True" in result.stdout


class TestDragModelNumerics:
    """Tests for correct numerical outputs from the drag model."""

    def test_drag_force_calculation(self, python_in_venv: Path):
        """Drag force should be calculated correctly: F = 0.5 * rho * v^2 * Cd * A."""
        # Expected: q = 0.5 * 1.225 * 20^2 = 245 Pa
        # F = 245 * 0.3 * 10 = 735 N
        code = textwrap.dedent("""
            from physics_models import create_drag
            model = create_drag(area=10.0, cd=0.3)
            result = model(rho=1.225, velocity=20.0)
            print(f"drag_force: {result.drag_force}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"

        # Parse the output and check the value
        for line in result.stdout.strip().split("\n"):
            if line.startswith("drag_force:"):
                drag_force = float(line.split(":")[1].strip())
                expected = 0.5 * 1.225 * 20.0 * 20.0 * 0.3 * 10.0  # = 735.0
                assert abs(drag_force - expected) < 1e-10, f"Expected {expected}, got {drag_force}"
                break
        else:
            pytest.fail("drag_force not found in output")

    def test_dynamic_pressure_calculation(self, python_in_venv: Path):
        """Dynamic pressure should be calculated correctly: q = 0.5 * rho * v^2."""
        # Expected: q = 0.5 * 1.225 * 20^2 = 245 Pa
        code = textwrap.dedent("""
            from physics_models import create_drag
            model = create_drag(area=10.0, cd=0.3)
            result = model(rho=1.225, velocity=20.0)
            print(f"dynamic_pressure: {result.dynamic_pressure}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"

        for line in result.stdout.strip().split("\n"):
            if line.startswith("dynamic_pressure:"):
                dynamic_pressure = float(line.split(":")[1].strip())
                expected = 0.5 * 1.225 * 20.0 * 20.0  # = 245.0
                assert (
                    abs(dynamic_pressure - expected) < 1e-10
                ), f"Expected {expected}, got {dynamic_pressure}"
                break
        else:
            pytest.fail("dynamic_pressure not found in output")

    def test_zero_velocity(self, python_in_venv: Path):
        """Zero velocity should produce zero drag and zero dynamic pressure."""
        code = textwrap.dedent("""
            from physics_models import create_drag
            model = create_drag(area=10.0, cd=0.3)
            result = model(rho=1.225, velocity=0.0)
            print(f"drag_force: {result.drag_force}")
            print(f"dynamic_pressure: {result.dynamic_pressure}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "drag_force: 0.0" in result.stdout
        assert "dynamic_pressure: 0.0" in result.stdout

    def test_different_parameters(self, python_in_venv: Path):
        """Different model parameters should produce different results."""
        code = textwrap.dedent("""
            from physics_models import create_drag

            # Two models with different drag coefficients
            model1 = create_drag(area=10.0, cd=0.3)
            model2 = create_drag(area=10.0, cd=0.6)

            result1 = model1(rho=1.225, velocity=20.0)
            result2 = model2(rho=1.225, velocity=20.0)

            # model2 should have double the drag force
            ratio = result2.drag_force / result1.drag_force
            print(f"ratio: {ratio}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "ratio: 2.0" in result.stdout


class TestDataclassBehavior:
    """Tests for dataclass behavior of generated types."""

    def test_params_is_frozen(self, python_in_venv: Path):
        """DragParams should be frozen (immutable)."""
        code = textwrap.dedent("""
            from physics_models import DragParams
            params = DragParams(area=10.0, cd=0.3)
            try:
                params.area = 20.0
                print("mutable")
            except Exception:
                print("frozen")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "frozen" in result.stdout

    def test_outputs_is_frozen(self, python_in_venv: Path):
        """DragOutputs should be frozen (immutable)."""
        code = textwrap.dedent("""
            from physics_models import create_drag
            model = create_drag(area=10.0, cd=0.3)
            result = model(rho=1.225, velocity=20.0)
            try:
                result.drag_force = 0.0
                print("mutable")
            except Exception:
                print("frozen")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "frozen" in result.stdout
