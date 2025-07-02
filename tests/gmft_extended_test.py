"""Extended tests for GMFT functionality."""

from __future__ import annotations

from dataclasses import replace
from typing import TYPE_CHECKING
from unittest.mock import Mock, patch

import pytest

from kreuzberg._gmft import (
    GMFTConfig,
    extract_tables_sync,
)
from kreuzberg._types import TableData
from kreuzberg.exceptions import MissingDependencyError

if TYPE_CHECKING:
    from pathlib import Path


def test_gmft_config_defaults() -> None:
    """Test GMFTConfig default values."""
    config = GMFTConfig()

    assert config.verbosity == 0
    assert config.formatter_base_threshold == 0.3
    assert config.detector_base_threshold == 0.9
    assert config.remove_null_rows is True
    assert config.apply_formatter_for_format_only is False
    assert config.visualize is False

    # Check cell_required_confidence defaults
    assert config.cell_required_confidence[0] == 0.3
    assert config.cell_required_confidence[4] == 0.5
    assert config.cell_required_confidence[6] == 99


def test_gmft_config_custom() -> None:
    """Test GMFTConfig with custom values."""
    config = GMFTConfig(
        verbosity=2,
        formatter_base_threshold=0.5,
        remove_null_rows=False,
        visualize=True,
    )

    assert config.verbosity == 2
    assert config.formatter_base_threshold == 0.5
    assert config.remove_null_rows is False
    assert config.visualize is True


def test_gmft_config_replace() -> None:
    """Test replacing GMFTConfig values."""
    config = GMFTConfig()
    new_config = replace(config, verbosity=3, visualize=True)

    assert config.verbosity == 0  # Original unchanged
    assert new_config.verbosity == 3
    assert new_config.visualize is True


def test_gmft_config_hash() -> None:
    """Test GMFTConfig is hashable."""
    config1 = GMFTConfig(verbosity=1)
    config2 = GMFTConfig(verbosity=1)
    config3 = GMFTConfig(verbosity=2)

    # Same configs should have same hash
    assert hash(config1) == hash(config2)
    # Different configs should have different hash
    assert hash(config1) != hash(config3)

    # Can be used in sets/dicts
    config_set = {config1, config2, config3}
    assert len(config_set) == 2  # config1 and config2 are the same


def test_extract_tables_sync_missing_dependency(tiny_pdf_with_tables: Path) -> None:
    """Test extract_tables_sync when gmft is not installed."""
    with patch("kreuzberg._gmft._import_gmft", side_effect=ImportError("No module named 'gmft'")):
        with pytest.raises(MissingDependencyError) as exc_info:
            extract_tables_sync(tiny_pdf_with_tables)

        assert "gmft" in str(exc_info.value)
        assert "table extraction" in str(exc_info.value)


def test_extract_tables_sync_success(tiny_pdf_with_tables: Path) -> None:
    """Test successful table extraction."""
    # Mock gmft
    mock_auto_table_detector = Mock()
    mock_pdf_handler = Mock()

    # Mock table detection
    mock_table = Mock()
    mock_table.bbox = [100, 200, 300, 400]
    mock_table.page_number = 1
    mock_table.confidence_score = 0.95

    mock_cropped_table = Mock()
    mock_cropped_table.df.to_dict.return_value = {
        "col1": [1, 2, 3],
        "col2": ["a", "b", "c"],
    }

    mock_auto_table_detector.return_value.extract.return_value = [(mock_table, mock_cropped_table)]

    with patch("kreuzberg._gmft._import_gmft") as mock_import:
        mock_import.return_value = (mock_auto_table_detector, mock_pdf_handler)

        results = extract_tables_sync(tiny_pdf_with_tables)

        assert len(results) == 1
        assert isinstance(results[0], TableData)
        assert results[0].page_number == 1
        assert results[0].confidence == 0.95
        assert results[0].data == {"col1": [1, 2, 3], "col2": ["a", "b", "c"]}


def test_extract_tables_sync_custom_config(tiny_pdf_with_tables: Path) -> None:
    """Test table extraction with custom config."""
    config = GMFTConfig(
        verbosity=2,
        detector_base_threshold=0.8,
        remove_null_rows=False,
    )

    mock_auto_table_detector = Mock()
    mock_pdf_handler = Mock()

    mock_auto_table_detector.return_value.extract.return_value = []

    with patch("kreuzberg._gmft._import_gmft") as mock_import:
        mock_import.return_value = (mock_auto_table_detector, mock_pdf_handler)

        extract_tables_sync(tiny_pdf_with_tables, config=config)

        # Verify config was passed
        mock_auto_table_detector.assert_called_once()
        call_kwargs = mock_auto_table_detector.call_args[1]
        assert call_kwargs["config"]["detector_base_threshold"] == 0.8
        assert call_kwargs["config"]["remove_null_rows"] is False


def test_extract_tables_sync_multiple_tables(tiny_pdf_with_tables: Path) -> None:
    """Test extraction of multiple tables."""
    mock_auto_table_detector = Mock()
    mock_pdf_handler = Mock()

    # Create multiple mock tables
    tables = []
    for i in range(3):
        mock_table = Mock()
        mock_table.bbox = [i * 100, i * 100, (i + 1) * 100, (i + 1) * 100]
        mock_table.page_number = i + 1
        mock_table.confidence_score = 0.9 - i * 0.1

        mock_cropped = Mock()
        mock_cropped.df.to_dict.return_value = {f"col{i}": [i, i + 1]}

        tables.append((mock_table, mock_cropped))

    mock_auto_table_detector.return_value.extract.return_value = tables

    with patch("kreuzberg._gmft._import_gmft") as mock_import:
        mock_import.return_value = (mock_auto_table_detector, mock_pdf_handler)

        results = extract_tables_sync(tiny_pdf_with_tables)

        assert len(results) == 3
        for i, table in enumerate(results):
            assert table.page_number == i + 1
            assert table.confidence == 0.9 - i * 0.1


def test_extract_tables_sync_no_tables(tiny_pdf_with_tables: Path) -> None:
    """Test extraction when no tables are found."""
    mock_auto_table_detector = Mock()
    mock_pdf_handler = Mock()

    mock_auto_table_detector.return_value.extract.return_value = []

    with patch("kreuzberg._gmft._import_gmft") as mock_import:
        mock_import.return_value = (mock_auto_table_detector, mock_pdf_handler)

        results = extract_tables_sync(tiny_pdf_with_tables)

        assert results == []


def test_extract_tables_sync_with_pages(tiny_pdf_with_tables: Path) -> None:
    """Test sync extraction with specific pages."""
    mock_auto_table_detector = Mock()
    mock_pdf_handler = Mock()

    mock_table = Mock()
    mock_table.bbox = [0, 0, 100, 100]
    mock_table.page_number = 1
    mock_table.confidence_score = 0.99

    mock_cropped = Mock()
    mock_cropped.df.to_dict.return_value = {"data": [1, 2, 3]}

    mock_auto_table_detector.return_value.extract.return_value = [(mock_table, mock_cropped)]

    with patch("kreuzberg._gmft._import_gmft") as mock_import:
        mock_import.return_value = (mock_auto_table_detector, mock_pdf_handler)

        extract_tables_sync(tiny_pdf_with_tables, pages=[1, 2, 3])

        # Verify pages were passed
        mock_auto_table_detector.return_value.extract.assert_called_once_with(
            pages=[0, 1, 2]  # GMFT uses 0-based indexing
        )
