"""Tests for generated Python packages."""

import subprocess
import textwrap
from pathlib import Path


class TestPackageImport:
    """Tests for basic package imports."""

    def test_import_package(self, python_in_venv: Path):
        """The minimal_example package should be importable."""
        code = "import minimal_example; print('OK')"
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Import failed:\n{result.stderr}"
        assert "OK" in result.stdout

    def test_import_create_scale(self, python_in_venv: Path):
        """create_scale should be importable from minimal_example."""
        code = "from minimal_example import create_scale; print('OK')"
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Import failed:\n{result.stderr}"
        assert "OK" in result.stdout

    def test_import_dataclasses(self, python_in_venv: Path):
        """ScaleParams and ScaleOutputs should be importable."""
        code = "from minimal_example import ScaleParams, ScaleOutputs; print('OK')"
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Import failed:\n{result.stderr}"
        assert "OK" in result.stdout


class TestModelAPI:
    """Tests for the model builder pattern API."""

    def test_create_scale_returns_callable(self, python_in_venv: Path):
        """create_scale should return a callable model function."""
        code = textwrap.dedent("""
            from minimal_example import create_scale
            model = create_scale(factor=2.0)
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
        """The model function should have a params attribute."""
        code = textwrap.dedent("""
            from minimal_example import create_scale
            model = create_scale(factor=2.5)
            print(f"has_params: {hasattr(model, 'params')}")
            print(f"factor: {model.params.factor}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "has_params: True" in result.stdout
        assert "factor: 2.5" in result.stdout

    def test_model_returns_outputs(self, python_in_venv: Path):
        """Calling the model should return a ScaleOutputs dataclass."""
        code = textwrap.dedent("""
            from minimal_example import create_scale, ScaleOutputs
            model = create_scale(factor=2.0)
            result = model(value=5.0)
            print(f"is_outputs: {isinstance(result, ScaleOutputs)}")
            print(f"has_scaled: {hasattr(result, 'scaled')}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "is_outputs: True" in result.stdout
        assert "has_scaled: True" in result.stdout


class TestNumericalCorrectness:
    """Tests for numerical correctness of model computations."""

    def test_scaling_calculation(self, python_in_venv: Path):
        """Test that scaling is computed correctly: scaled = value * factor."""
        code = textwrap.dedent("""
            from minimal_example import create_scale
            model = create_scale(factor=2.5)
            result = model(value=4.0)

            expected = 4.0 * 2.5
            actual = result.scaled

            print(f"expected: {expected}")
            print(f"actual: {actual}")
            print(f"match: {abs(expected - actual) < 0.0001}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "expected: 10.0" in result.stdout
        assert "actual: 10.0" in result.stdout
        assert "match: True" in result.stdout

    def test_zero_factor(self, python_in_venv: Path):
        """Test with factor = 0."""
        code = textwrap.dedent("""
            from minimal_example import create_scale
            model = create_scale(factor=0.0)
            result = model(value=100.0)
            print(f"scaled: {result.scaled}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "scaled: 0.0" in result.stdout

    def test_negative_values(self, python_in_venv: Path):
        """Test with negative values."""
        code = textwrap.dedent("""
            from minimal_example import create_scale
            model = create_scale(factor=-2.0)
            result = model(value=5.0)
            print(f"scaled: {result.scaled}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "scaled: -10.0" in result.stdout

    def test_different_parameters(self, python_in_venv: Path):
        """Test that different model instances have independent parameters."""
        code = textwrap.dedent("""
            from minimal_example import create_scale

            # Two models with different factors
            model1 = create_scale(factor=2.0)
            model2 = create_scale(factor=3.0)

            result1 = model1(value=10.0)
            result2 = model2(value=10.0)

            # model2 should scale 1.5x more than model1
            ratio = result2.scaled / result1.scaled
            print(f"ratio: {ratio}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "ratio: 1.5" in result.stdout


class TestDataclassBehavior:
    """Tests for dataclass behavior of params and outputs."""

    def test_params_is_frozen(self, python_in_venv: Path):
        """ScaleParams should be frozen (immutable)."""
        code = textwrap.dedent("""
            from minimal_example import ScaleParams
            params = ScaleParams(factor=2.0)
            try:
                params.factor = 3.0
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
        """ScaleOutputs should be frozen (immutable)."""
        code = textwrap.dedent("""
            from minimal_example import create_scale
            model = create_scale(factor=2.0)
            result = model(value=5.0)
            try:
                result.scaled = 0.0
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
