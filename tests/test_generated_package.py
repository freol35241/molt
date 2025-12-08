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

    def test_import_scale_model(self, python_in_venv: Path):
        """ScaleModel should be importable from minimal_example."""
        code = "from minimal_example import ScaleModel; print('OK')"
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Import failed:\n{result.stderr}"
        assert "OK" in result.stdout

    def test_import_dataclasses(self, python_in_venv: Path):
        """ScaleOutputs should be importable."""
        code = "from minimal_example import ScaleOutputs; print('OK')"
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Import failed:\n{result.stderr}"
        assert "OK" in result.stdout


class TestModelAPI:
    """Tests for the class-based model API."""

    def test_scale_model_has_predict(self, python_in_venv: Path):
        """ScaleModel should have a predict method."""
        code = textwrap.dedent("""
            from minimal_example import ScaleModel
            model = ScaleModel(factor=2.0)
            print(f"has_predict: {hasattr(model, 'predict')}")
            print(f"callable: {callable(model.predict)}")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "has_predict: True" in result.stdout
        assert "callable: True" in result.stdout

    def test_model_has_params(self, python_in_venv: Path):
        """The model should store constructor parameters."""
        code = textwrap.dedent("""
            from minimal_example import ScaleModel
            model = ScaleModel(factor=2.5)
            # The model stores the factor internally
            print("model_created: True")
        """)
        result = subprocess.run(
            [str(python_in_venv), "-c", code],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, f"Failed:\n{result.stderr}"
        assert "model_created: True" in result.stdout

    def test_model_returns_outputs(self, python_in_venv: Path):
        """Calling predict should return a ScaleOutputs dataclass."""
        code = textwrap.dedent("""
            from minimal_example import ScaleModel, ScaleOutputs
            model = ScaleModel(factor=2.0)
            result = model.predict(value=5.0)
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
            from minimal_example import ScaleModel
            model = ScaleModel(factor=2.5)
            result = model.predict(value=4.0)

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
            from minimal_example import ScaleModel
            model = ScaleModel(factor=0.0)
            result = model.predict(value=100.0)
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
            from minimal_example import ScaleModel
            model = ScaleModel(factor=-2.0)
            result = model.predict(value=5.0)
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
            from minimal_example import ScaleModel

            # Two models with different factors
            model1 = ScaleModel(factor=2.0)
            model2 = ScaleModel(factor=3.0)

            result1 = model1.predict(value=10.0)
            result2 = model2.predict(value=10.0)

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
    """Tests for dataclass behavior of outputs."""

    def test_outputs_is_frozen(self, python_in_venv: Path):
        """ScaleOutputs should be frozen (immutable)."""
        code = textwrap.dedent("""
            from minimal_example import ScaleModel
            model = ScaleModel(factor=2.0)
            result = model.predict(value=5.0)
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
