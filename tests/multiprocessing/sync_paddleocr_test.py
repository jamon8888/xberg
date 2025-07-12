"""Tests for synchronous PaddleOCR multiprocessing utilities."""

from __future__ import annotations

import tempfile
from pathlib import Path
from typing import TYPE_CHECKING, Any, cast
from unittest.mock import Mock, patch

import pytest
from PIL import Image

from kreuzberg._mime_types import PLAIN_TEXT_MIME_TYPE
from kreuzberg._ocr._paddleocr import PaddleOCRConfig
from kreuzberg._ocr._sync import (
    _get_paddleocr_instance,
    process_batch_images_sync,
    process_batch_images_threaded,
    process_image_bytes_paddleocr_sync,
    process_image_paddleocr_sync,
)
from kreuzberg._types import ExtractionResult
from kreuzberg.exceptions import MissingDependencyError, OCRError

if TYPE_CHECKING:
    from collections.abc import Generator


@pytest.fixture
def sample_image_path() -> Generator[str, None, None]:
    """Create a sample image file for testing."""
    with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as tmp_file:
        # Create a simple white image
        image = Image.new("RGB", (100, 50), color="white")
        image.save(tmp_file.name, format="PNG")
        yield tmp_file.name

    # Cleanup
    Path(tmp_file.name).unlink(missing_ok=True)


@pytest.fixture
def sample_image_bytes() -> bytes:
    """Create sample image bytes for testing."""
    import io

    image = Image.new("RGB", (100, 50), color="white")
    img_bytes = io.BytesIO()
    image.save(img_bytes, format="PNG")
    return img_bytes.getvalue()


@pytest.fixture
def mock_paddleocr() -> Generator[Mock, None, None]:
    """Mock PaddleOCR module and PaddleOCR class."""
    mock_paddleocr_module = Mock()
    mock_ocr_instance = Mock()

    # Mock the OCR result structure for PaddleOCR
    mock_result = Mock()
    mock_result.json = {"res": {"rec_texts": ["Hello", "World"], "rec_scores": [0.95, 0.88]}}
    mock_ocr_instance.ocr.return_value = [mock_result]

    mock_paddleocr_module.PaddleOCR.return_value = mock_ocr_instance

    with patch.dict("sys.modules", {"paddleocr": mock_paddleocr_module}):
        yield mock_ocr_instance


def test_get_paddleocr_instance_missing_dependency() -> None:
    """Test that MissingDependencyError is raised when PaddleOCR is not installed."""
    config = PaddleOCRConfig()

    # Mock the import to simulate missing paddleocr
    import builtins

    original_import = builtins.__import__

    def mock_import(name: str, *args: Any, **kwargs: Any) -> Any:
        if name == "paddleocr":
            raise ImportError("No module named 'paddleocr'")
        return original_import(name, *args, **kwargs)

    with patch("builtins.__import__", side_effect=mock_import):
        with pytest.raises(MissingDependencyError, match="PaddleOCR is not installed"):
            _get_paddleocr_instance(config)


def test_get_paddleocr_instance_basic_config() -> None:
    """Test PaddleOCR instance creation with basic config."""
    config = PaddleOCRConfig()

    mock_paddleocr_module = Mock()
    mock_ocr_instance = Mock()
    mock_paddleocr_module.PaddleOCR.return_value = mock_ocr_instance

    with patch.dict("sys.modules", {"paddleocr": mock_paddleocr_module}):
        result = _get_paddleocr_instance(config)

        assert result == mock_ocr_instance
        # PaddleOCRConfig has defaults for all parameters, so they'll be included
        expected_kwargs = {
            "lang": "en",
            "use_textline_orientation": True,
            "text_det_thresh": 0.3,
            "text_det_box_thresh": 0.5,
            "text_det_unclip_ratio": 2.0,
            "text_det_limit_side_len": 960,
            "text_rec_score_thresh": 0.5,
        }
        mock_paddleocr_module.PaddleOCR.assert_called_once_with(**expected_kwargs)


def test_get_paddleocr_instance_device_config() -> None:
    """Test PaddleOCR instance creation with device config."""
    config = PaddleOCRConfig(device="cuda")

    mock_paddleocr_module = Mock()
    mock_ocr_instance = Mock()
    mock_paddleocr_module.PaddleOCR.return_value = mock_ocr_instance

    with patch.dict("sys.modules", {"paddleocr": mock_paddleocr_module}):
        result = _get_paddleocr_instance(config)

        assert result == mock_ocr_instance
        expected_kwargs = {
            "lang": "en",
            "use_textline_orientation": True,
            "text_det_thresh": 0.3,
            "text_det_box_thresh": 0.5,
            "text_det_unclip_ratio": 2.0,
            "text_det_limit_side_len": 960,
            "text_rec_score_thresh": 0.5,
        }
        mock_paddleocr_module.PaddleOCR.assert_called_once_with(**expected_kwargs)


def test_get_paddleocr_instance_cpu_device() -> None:
    """Test PaddleOCR instance creation with CPU device."""
    config = PaddleOCRConfig(device="cpu")

    mock_paddleocr_module = Mock()
    mock_ocr_instance = Mock()
    mock_paddleocr_module.PaddleOCR.return_value = mock_ocr_instance

    with patch.dict("sys.modules", {"paddleocr": mock_paddleocr_module}):
        result = _get_paddleocr_instance(config)

        assert result == mock_ocr_instance
        expected_kwargs = {
            "lang": "en",
            "use_textline_orientation": True,
            "text_det_thresh": 0.3,
            "text_det_box_thresh": 0.5,
            "text_det_unclip_ratio": 2.0,
            "text_det_limit_side_len": 960,
            "text_rec_score_thresh": 0.5,
        }
        mock_paddleocr_module.PaddleOCR.assert_called_once_with(**expected_kwargs)


def test_get_paddleocr_instance_legacy_use_gpu() -> None:
    """Test PaddleOCR instance creation with legacy use_gpu config."""
    # Create a config without device attribute to test use_gpu fallback
    config = Mock()
    config.language = "en"
    config.use_angle_cls = True

    # Mock hasattr to explicitly control what attributes are detected
    def mock_hasattr(obj: Any, attr: str) -> bool:
        if obj is config:
            return attr in ["language", "use_angle_cls"]
        return hasattr(obj, attr)

    mock_paddleocr_module = Mock()
    mock_ocr_instance = Mock()
    mock_paddleocr_module.PaddleOCR.return_value = mock_ocr_instance

    with patch.dict("sys.modules", {"paddleocr": mock_paddleocr_module}):
        with patch("builtins.hasattr", side_effect=mock_hasattr):
            result = _get_paddleocr_instance(config)

            assert result == mock_ocr_instance
            # Should call with minimal args since this mock doesn't have the default attributes
            mock_paddleocr_module.PaddleOCR.assert_called_once_with(lang="en", use_textline_orientation=True)


def test_get_paddleocr_instance_with_advanced_config() -> None:
    """Test PaddleOCR instance creation with advanced configuration options."""
    config = Mock()
    config.language = "en"
    config.use_angle_cls = True
    config.det_db_thresh = 0.3
    config.det_db_box_thresh = 0.6
    config.det_db_unclip_ratio = 1.5
    config.det_max_side_len = 960
    config.drop_score = 0.5

    mock_paddleocr_module = Mock()
    mock_ocr_instance = Mock()
    mock_paddleocr_module.PaddleOCR.return_value = mock_ocr_instance

    with patch.dict("sys.modules", {"paddleocr": mock_paddleocr_module}):
        result = _get_paddleocr_instance(config)

        assert result == mock_ocr_instance
        mock_paddleocr_module.PaddleOCR.assert_called_once_with(
            lang="en",
            use_textline_orientation=True,
            text_det_thresh=0.3,
            text_det_box_thresh=0.6,
            text_det_unclip_ratio=1.5,
            text_det_limit_side_len=960,
            text_rec_score_thresh=0.5,
        )


def test_process_image_paddleocr_sync_success(sample_image_path: str, mock_paddleocr: Mock) -> None:
    """Test successful image processing with sync pure implementation."""
    config = PaddleOCRConfig()
    result = process_image_paddleocr_sync(sample_image_path, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == "Hello\nWorld"
    assert result.mime_type == PLAIN_TEXT_MIME_TYPE
    assert result.metadata == {"confidence": 0.915}  # (0.95 + 0.88) / 2
    assert result.chunks == []

    mock_paddleocr.ocr.assert_called_once_with(str(sample_image_path))


def test_process_image_paddleocr_sync_no_results(sample_image_path: str, mock_paddleocr: Mock) -> None:
    """Test image processing when PaddleOCR returns no results."""
    mock_paddleocr.ocr.return_value = []

    config = PaddleOCRConfig()
    result = process_image_paddleocr_sync(sample_image_path, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == ""
    assert result.mime_type == PLAIN_TEXT_MIME_TYPE
    assert result.metadata == {}
    assert result.chunks == []


def test_process_image_paddleocr_sync_empty_results(sample_image_path: str, mock_paddleocr: Mock) -> None:
    """Test image processing when PaddleOCR returns empty results."""
    mock_paddleocr.ocr.return_value = [None]

    config = PaddleOCRConfig()
    result = process_image_paddleocr_sync(sample_image_path, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == ""
    assert result.mime_type == PLAIN_TEXT_MIME_TYPE
    assert result.metadata == {}
    assert result.chunks == []


def test_process_image_paddleocr_sync_no_texts(sample_image_path: str, mock_paddleocr: Mock) -> None:
    """Test image processing when OCR result has no texts."""
    mock_result = Mock()
    mock_result.json = {"res": {"rec_texts": [], "rec_scores": []}}
    mock_paddleocr.ocr.return_value = [mock_result]

    config = PaddleOCRConfig()
    result = process_image_paddleocr_sync(sample_image_path, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == ""
    assert result.mime_type == PLAIN_TEXT_MIME_TYPE
    assert result.metadata == {}
    assert result.chunks == []


def test_process_image_paddleocr_sync_missing_rec_texts(sample_image_path: str, mock_paddleocr: Mock) -> None:
    """Test image processing when OCR result is missing rec_texts."""
    mock_result = Mock()
    mock_result.json = {"res": {"rec_scores": [0.9]}}
    mock_paddleocr.ocr.return_value = [mock_result]

    config = PaddleOCRConfig()
    result = process_image_paddleocr_sync(sample_image_path, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == ""
    assert result.mime_type == PLAIN_TEXT_MIME_TYPE
    assert result.metadata == {}
    assert result.chunks == []


def test_process_image_paddleocr_sync_no_scores(sample_image_path: str, mock_paddleocr: Mock) -> None:
    """Test image processing when OCR result has no scores."""
    mock_result = Mock()
    mock_result.json = {"res": {"rec_texts": ["Hello", "World"], "rec_scores": []}}
    mock_paddleocr.ocr.return_value = [mock_result]

    config = PaddleOCRConfig()
    result = process_image_paddleocr_sync(sample_image_path, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == "Hello\nWorld"
    assert result.mime_type == PLAIN_TEXT_MIME_TYPE
    assert result.metadata == {}  # No confidence when no scores
    assert result.chunks == []


def test_process_image_paddleocr_sync_default_config(sample_image_path: str, mock_paddleocr: Mock) -> None:
    """Test image processing with default config (None)."""
    result = process_image_paddleocr_sync(sample_image_path, None)

    assert isinstance(result, ExtractionResult)
    assert result.content == "Hello\nWorld"
    assert result.metadata == {"confidence": 0.915}


def test_process_image_paddleocr_sync_exception(sample_image_path: str) -> None:
    """Test OCRError is raised when PaddleOCR processing fails."""
    config = PaddleOCRConfig()

    with patch("kreuzberg._ocr._sync._get_paddleocr_instance", side_effect=Exception("PaddleOCR failed")):
        with pytest.raises(OCRError, match="PaddleOCR processing failed"):
            process_image_paddleocr_sync(sample_image_path, config)


def test_process_image_bytes_paddleocr_sync_success(sample_image_bytes: bytes, mock_paddleocr: Mock) -> None:
    """Test successful image bytes processing."""
    config = PaddleOCRConfig()
    result = process_image_bytes_paddleocr_sync(sample_image_bytes, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == "Hello\nWorld"
    assert result.metadata == {"confidence": 0.915}


def test_process_image_bytes_paddleocr_sync_cleanup() -> None:
    """Test that temporary files are cleaned up after processing."""
    sample_bytes = b"fake image data"

    with patch("kreuzberg._ocr._sync.Image.open") as mock_open:
        mock_image = Mock()
        mock_open.return_value.__enter__.return_value = mock_image

        with patch("kreuzberg._ocr._sync.process_image_paddleocr_sync") as mock_process:
            mock_process.return_value = ExtractionResult(
                content="test", mime_type=PLAIN_TEXT_MIME_TYPE, metadata={}, chunks=[]
            )

            # Track created temp files
            created_files = []

            # Create a real temporary file and track it
            import tempfile as real_tempfile

            original_named_temp_file = real_tempfile.NamedTemporaryFile

            def track_tempfile(*args: Any, **kwargs: Any) -> Any:
                tf = original_named_temp_file(*args, **kwargs)
                created_files.append(tf.name)
                return tf

            with patch("tempfile.NamedTemporaryFile", side_effect=track_tempfile):
                result = process_image_bytes_paddleocr_sync(sample_bytes, None)

                assert isinstance(result, ExtractionResult)
                assert result.content == "test"

                # Verify temp files were cleaned up
                for temp_file in created_files:
                    assert not Path(temp_file).exists()


def test_process_batch_images_sync_success(mock_paddleocr: Mock) -> None:
    """Test successful batch processing with sync pure implementation."""
    # Create temporary image files
    image_paths = []
    for i in range(3):
        with tempfile.NamedTemporaryFile(suffix=f"_{i}.png", delete=False) as tmp_file:
            image = Image.new("RGB", (100, 50), color="white")
            image.save(tmp_file.name, format="PNG")
            image_paths.append(tmp_file.name)

    try:
        # Mock different results for each image based on path
        def side_effect(path: Any, **kwargs: Any) -> Any:
            for i, img_path in enumerate(image_paths):
                if str(img_path) == str(path):
                    mock_result = Mock()
                    mock_result.json = {"res": {"rec_texts": [f"Image {i}"], "rec_scores": [0.9]}}
                    return [mock_result]
            # Default fallback
            mock_result = Mock()
            mock_result.json = {"res": {"rec_texts": ["Default"], "rec_scores": [0.9]}}
            return [mock_result]

        mock_paddleocr.ocr.side_effect = side_effect

        config = PaddleOCRConfig()
        results = process_batch_images_sync(cast("list[str | Path]", image_paths), config, backend="paddleocr")

        assert len(results) == 3
        for i, result in enumerate(results):
            assert isinstance(result, ExtractionResult)
            assert result.content == f"Image {i}"
            assert result.metadata == {"confidence": 0.9}

    finally:
        # Cleanup temp files
        for path in image_paths:
            Path(path).unlink(missing_ok=True)


def test_process_batch_images_sync_empty_list() -> None:
    """Test batch processing with empty image list."""
    results = process_batch_images_sync([], None, backend="paddleocr")
    assert results == []


def test_process_batch_images_threaded_success(mock_paddleocr: Mock) -> None:
    """Test successful threaded batch processing."""
    # Create temporary image files
    image_paths = []
    for i in range(3):
        with tempfile.NamedTemporaryFile(suffix=f"_{i}.png", delete=False) as tmp_file:
            image = Image.new("RGB", (100, 50), color="white")
            image.save(tmp_file.name, format="PNG")
            image_paths.append(tmp_file.name)

    try:
        # Mock results based on image path to ensure correct ordering
        def side_effect(path: Any, **kwargs: Any) -> Any:
            for i, img_path in enumerate(image_paths):
                if str(img_path) == str(path):
                    mock_result = Mock()
                    mock_result.json = {"res": {"rec_texts": [f"Threaded {i}"], "rec_scores": [0.8]}}
                    return [mock_result]
            # Default fallback
            mock_result = Mock()
            mock_result.json = {"res": {"rec_texts": ["Default"], "rec_scores": [0.8]}}
            return [mock_result]

        mock_paddleocr.ocr.side_effect = side_effect

        config = PaddleOCRConfig()
        results = process_batch_images_threaded(cast("list[str | Path]", image_paths), config, max_workers=2)

        assert len(results) == 3
        # Results should be in same order as input
        for i, result in enumerate(results):
            assert isinstance(result, ExtractionResult)
            assert result.content == f"Threaded {i}"
            assert result.metadata == {"confidence": 0.8}

    finally:
        # Cleanup temp files
        for path in image_paths:
            Path(path).unlink(missing_ok=True)


def test_process_batch_images_threaded_with_errors(mock_paddleocr: Mock) -> None:
    """Test threaded batch processing with some errors."""
    # Create temporary image files
    image_paths = []
    for i in range(3):
        with tempfile.NamedTemporaryFile(suffix=f"_{i}.png", delete=False) as tmp_file:
            image = Image.new("RGB", (100, 50), color="white")
            image.save(tmp_file.name, format="PNG")
            image_paths.append(tmp_file.name)

    try:
        # Mock mixed results (success, error, success)
        def side_effect(path: Any, **kwargs: Any) -> Any:
            if "1.png" in str(path):
                raise Exception("Processing failed")
            mock_result = Mock()
            mock_result.json = {"res": {"rec_texts": ["Success"], "rec_scores": [0.9]}}
            return [mock_result]

        mock_paddleocr.ocr.side_effect = side_effect

        config = PaddleOCRConfig()
        results = process_batch_images_threaded(cast("list[str | Path]", image_paths), config, max_workers=2)

        assert len(results) == 3
        assert isinstance(results[0], ExtractionResult)
        assert results[0].content == "Success"

        # Middle result should be error
        assert isinstance(results[1], ExtractionResult)
        assert "Error:" in results[1].content
        assert "error" in results[1].metadata

        assert isinstance(results[2], ExtractionResult)
        assert results[2].content == "Success"

    finally:
        # Cleanup temp files
        for path in image_paths:
            Path(path).unlink(missing_ok=True)


def test_process_batch_images_threaded_default_workers(mock_paddleocr: Mock) -> None:
    """Test threaded batch processing with default worker count."""
    image_paths = ["image1.png", "image2.png"]  # Fake paths for this test

    with patch("kreuzberg._ocr._sync.process_image_paddleocr_sync") as mock_process:
        mock_process.return_value = ExtractionResult(
            content="test", mime_type=PLAIN_TEXT_MIME_TYPE, metadata={}, chunks=[]
        )

        # Patch multiprocessing.cpu_count inside the function
        with patch("multiprocessing.cpu_count", return_value=8):
            results = process_batch_images_threaded(
                cast("list[str | Path]", image_paths), None, backend="paddleocr", max_workers=None
            )

            assert len(results) == 2
            # Should use min(len(image_paths), cpu_count()) = min(2, 8) = 2
            assert mock_process.call_count == 2


def test_process_batch_images_threaded_empty_list() -> None:
    """Test threaded batch processing with empty image list."""
    results = process_batch_images_threaded([], None, max_workers=2)
    assert results == []
