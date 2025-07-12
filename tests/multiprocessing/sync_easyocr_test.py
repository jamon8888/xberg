"""Tests for synchronous EasyOCR multiprocessing utilities."""

from __future__ import annotations

import tempfile
from pathlib import Path
from typing import TYPE_CHECKING, Any, cast
from unittest.mock import Mock, patch

import pytest
from PIL import Image

from kreuzberg._mime_types import PLAIN_TEXT_MIME_TYPE
from kreuzberg._ocr._easyocr import EasyOCRConfig
from kreuzberg._ocr._sync import (
    _get_easyocr_instance,
    process_batch_images_sync,
    process_batch_images_threaded,
    process_image_bytes_easyocr_sync,
    process_image_easyocr_sync,
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
def mock_easyocr() -> Generator[Mock, None, None]:
    """Mock EasyOCR module and Reader class."""
    mock_easyocr_module = Mock()
    mock_reader = Mock()
    mock_easyocr_module.Reader.return_value = mock_reader

    with patch.dict("sys.modules", {"easyocr": mock_easyocr_module}):
        yield mock_reader


def test_get_easyocr_instance_missing_dependency() -> None:
    """Test that MissingDependencyError is raised when EasyOCR is not installed."""
    config = EasyOCRConfig()

    # Mock the import to simulate missing easyocr
    import builtins

    original_import = builtins.__import__

    def mock_import(name: str, *args: Any, **kwargs: Any) -> Any:
        if name == "easyocr":
            raise ImportError("No module named 'easyocr'")
        return original_import(name, *args, **kwargs)

    with patch("builtins.__import__", side_effect=mock_import):
        with pytest.raises(MissingDependencyError, match="EasyOCR is not installed"):
            _get_easyocr_instance(config)


def test_get_easyocr_instance_basic_config() -> None:
    """Test EasyOCR instance creation with basic config."""
    config = EasyOCRConfig()

    mock_easyocr_module = Mock()
    mock_reader = Mock()
    mock_easyocr_module.Reader.return_value = mock_reader

    with patch.dict("sys.modules", {"easyocr": mock_easyocr_module}):
        result = _get_easyocr_instance(config)

        assert result == mock_reader
        # Default EasyOCRConfig has device="auto" which is treated as GPU
        mock_easyocr_module.Reader.assert_called_once_with(lang_list=["en"], gpu=True, verbose=False)


def test_get_easyocr_instance_gpu_device_config() -> None:
    """Test EasyOCR instance creation with GPU device config."""
    config = EasyOCRConfig(device="cuda")

    mock_easyocr_module = Mock()
    mock_reader = Mock()
    mock_easyocr_module.Reader.return_value = mock_reader

    with patch.dict("sys.modules", {"easyocr": mock_easyocr_module}):
        result = _get_easyocr_instance(config)

        assert result == mock_reader
        mock_easyocr_module.Reader.assert_called_once_with(lang_list=["en"], gpu=True, verbose=False)


def test_get_easyocr_instance_cpu_device_config() -> None:
    """Test EasyOCR instance creation with CPU device config."""
    config = EasyOCRConfig(device="cpu")

    mock_easyocr_module = Mock()
    mock_reader = Mock()
    mock_easyocr_module.Reader.return_value = mock_reader

    with patch.dict("sys.modules", {"easyocr": mock_easyocr_module}):
        result = _get_easyocr_instance(config)

        assert result == mock_reader
        mock_easyocr_module.Reader.assert_called_once_with(lang_list=["en"], gpu=False, verbose=False)


def test_get_easyocr_instance_legacy_use_gpu_config() -> None:
    """Test EasyOCR instance creation with legacy use_gpu config."""
    # Create a config without device attribute to test use_gpu fallback
    config = Mock()
    config.use_gpu = True  # Legacy attribute
    config.language = "en"  # Add language attribute
    # Set default None values for all optional attributes
    config.model_storage_directory = None
    config.user_network_directory = None
    config.recog_network = None
    config.detector = None
    config.recognizer = None
    config.quantize = None
    config.cudnn_benchmark = None
    # Don't have device attribute

    mock_easyocr_module = Mock()
    mock_reader = Mock()
    mock_easyocr_module.Reader.return_value = mock_reader

    with patch.dict("sys.modules", {"easyocr": mock_easyocr_module}):
        result = _get_easyocr_instance(config)

        assert result == mock_reader
        mock_easyocr_module.Reader.assert_called_once_with(lang_list=["en"], gpu=True, verbose=False)


def test_get_easyocr_instance_multiple_languages() -> None:
    """Test EasyOCR instance creation with multiple languages."""
    config = EasyOCRConfig(language="en,fr,de")

    mock_easyocr_module = Mock()
    mock_reader = Mock()
    mock_easyocr_module.Reader.return_value = mock_reader

    with patch.dict("sys.modules", {"easyocr": mock_easyocr_module}):
        result = _get_easyocr_instance(config)

        assert result == mock_reader
        # Default EasyOCRConfig has device="auto" which is treated as GPU
        mock_easyocr_module.Reader.assert_called_once_with(lang_list=["en", "fr", "de"], gpu=True, verbose=False)


def test_get_easyocr_instance_language_list() -> None:
    """Test EasyOCR instance creation with language list."""
    config = EasyOCRConfig(language=["en", "FR", "De"])

    mock_easyocr_module = Mock()
    mock_reader = Mock()
    mock_easyocr_module.Reader.return_value = mock_reader

    with patch.dict("sys.modules", {"easyocr": mock_easyocr_module}):
        result = _get_easyocr_instance(config)

        assert result == mock_reader
        # Default EasyOCRConfig has device="auto" which is treated as GPU
        mock_easyocr_module.Reader.assert_called_once_with(lang_list=["en", "fr", "de"], gpu=True, verbose=False)


@pytest.mark.skip("Complex mocking leads to issues")
def test_get_easyocr_instance_advanced_config() -> None:
    """Test EasyOCR instance creation with advanced configuration options."""
    config = EasyOCRConfig()

    mock_easyocr_module = Mock()
    mock_reader = Mock()
    mock_easyocr_module.Reader.return_value = mock_reader

    with patch.dict("sys.modules", {"easyocr": mock_easyocr_module}):
        # Mock getattr to return advanced config values
        with patch("builtins.getattr") as mock_getattr:

            def getattr_side_effect(obj: Any, attr: str, default: Any = None) -> Any:
                advanced_attrs = {
                    "model_storage_directory": "/path/to/models",
                    "user_network_directory": "/path/to/networks",
                    "recog_network": "custom_network",
                    "detector": "custom_detector",
                    "recognizer": "custom_recognizer",
                    "quantize": True,
                    "cudnn_benchmark": True,
                }
                if attr in advanced_attrs:
                    return advanced_attrs[attr]
                # Use actual getattr for other attributes
                try:
                    return getattr(type(obj), attr, default)
                except AttributeError:
                    return default

            mock_getattr.side_effect = getattr_side_effect

            result = _get_easyocr_instance(config)

            assert result == mock_reader
            expected_call_kwargs = {
                "lang_list": ["en"],
                "gpu": True,  # Default device="auto" treated as GPU
                "model_storage_directory": "/path/to/models",
                "user_network_directory": "/path/to/networks",
                "recog_network": "custom_network",
                "detector": "custom_detector",
                "recognizer": "custom_recognizer",
                "verbose": False,
                "quantize": True,
                "cudnn_benchmark": True,
            }
            mock_easyocr_module.Reader.assert_called_once_with(**expected_call_kwargs)


def test_process_image_easyocr_sync_success(sample_image_path: str, mock_easyocr: Mock) -> None:
    """Test successful image processing with sync pure implementation."""
    # Mock EasyOCR results
    mock_results = [
        ([[0, 0], [100, 0], [100, 20], [0, 20]], "Hello", 0.95),
        ([[0, 25], [100, 25], [100, 45], [0, 45]], "World", 0.88),
    ]
    mock_easyocr.readtext.return_value = mock_results

    config = EasyOCRConfig()
    result = process_image_easyocr_sync(sample_image_path, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == "Hello\nWorld"
    assert result.mime_type == PLAIN_TEXT_MIME_TYPE
    assert result.metadata == {"confidence": 0.915}  # (0.95 + 0.88) / 2
    assert result.chunks == []

    mock_easyocr.readtext.assert_called_once()


def test_process_image_easyocr_sync_no_results(sample_image_path: str, mock_easyocr: Mock) -> None:
    """Test image processing when EasyOCR returns no results."""
    mock_easyocr.readtext.return_value = []

    config = EasyOCRConfig()
    result = process_image_easyocr_sync(sample_image_path, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == ""
    assert result.mime_type == PLAIN_TEXT_MIME_TYPE
    assert result.metadata == {}
    assert result.chunks == []


def test_process_image_easyocr_sync_default_config(sample_image_path: str, mock_easyocr: Mock) -> None:
    """Test image processing with default config (None)."""
    mock_easyocr.readtext.return_value = [([[0, 0], [100, 0], [100, 20], [0, 20]], "Test", 0.9)]

    result = process_image_easyocr_sync(sample_image_path, None)

    assert isinstance(result, ExtractionResult)
    assert result.content == "Test"
    assert result.metadata == {"confidence": 0.9}


def test_process_image_easyocr_sync_detail_false(sample_image_path: str, mock_easyocr: Mock) -> None:
    """Test image processing with detail=False config."""
    mock_easyocr.readtext.return_value = ["Hello", "World"]

    # Create a config with all needed attributes
    config = Mock()
    config.decoder = "greedy"
    config.beam_width = 5
    config.rotation_info = [0]
    config.min_size = 20
    config.text_threshold = 0.7
    config.low_text = 0.4
    config.link_threshold = 0.4
    config.canvas_size = 2560
    config.mag_ratio = 1.0
    config.slope_ths = 0.1
    config.ycenter_ths = 0.5
    config.height_ths = 0.7
    config.width_ths = 0.5
    config.add_margin = 0.1
    config.x_ths = 1.0
    config.y_ths = 0.5
    config.detail = False  # This is what we want to test
    config.device = None
    config.language = "en"
    # Set all optional attributes to None
    config.model_storage_directory = None
    config.user_network_directory = None
    config.recog_network = None
    config.detector = None
    config.recognizer = None
    config.quantize = None
    config.cudnn_benchmark = None
    config.batch_size = 1
    config.workers = 0
    config.allowlist = None
    config.blocklist = None
    config.paragraph = False

    result = process_image_easyocr_sync(sample_image_path, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == "Hello\nWorld"
    assert result.metadata == {"confidence": 1.0}  # Default confidence when detail=False


def test_process_image_easyocr_sync_short_result(sample_image_path: str, mock_easyocr: Mock) -> None:
    """Test image processing with short result (missing confidence)."""
    # Result with only 2 elements (no confidence)
    mock_results = [([[0, 0], [100, 0], [100, 20], [0, 20]], "Hello")]
    mock_easyocr.readtext.return_value = mock_results

    config = EasyOCRConfig()
    result = process_image_easyocr_sync(sample_image_path, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == "Hello"
    assert result.metadata == {"confidence": 1.0}  # Default confidence


def test_process_image_easyocr_sync_invalid_result(sample_image_path: str, mock_easyocr: Mock) -> None:
    """Test image processing with invalid result structure."""
    # Result with only 1 element (invalid)
    mock_results = [([[0, 0], [100, 0], [100, 20], [0, 20]],)]
    mock_easyocr.readtext.return_value = mock_results

    config = EasyOCRConfig()
    result = process_image_easyocr_sync(sample_image_path, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == ""
    assert result.metadata == {}


def test_process_image_easyocr_sync_exception(sample_image_path: str) -> None:
    """Test OCRError is raised when EasyOCR processing fails."""
    config = EasyOCRConfig()

    with patch("kreuzberg._ocr._sync._get_easyocr_instance", side_effect=Exception("EasyOCR failed")):
        with pytest.raises(OCRError, match="EasyOCR processing failed"):
            process_image_easyocr_sync(sample_image_path, config)


@pytest.mark.skip("Complex mocking leads to issues")
def test_process_image_easyocr_sync_advanced_config(sample_image_path: str, mock_easyocr: Mock) -> None:
    """Test image processing with advanced EasyOCR configuration."""
    mock_easyocr.readtext.return_value = [([[0, 0], [100, 0], [100, 20], [0, 20]], "Test", 0.9)]

    config = EasyOCRConfig(
        decoder="beamsearch",
        beam_width=10,
        rotation_info=[0, 90, 180, 270],
        min_size=20,
        text_threshold=0.7,
        low_text=0.4,
        link_threshold=0.4,
        canvas_size=2560,
        mag_ratio=1.0,
        slope_ths=0.1,
        ycenter_ths=0.5,
        height_ths=0.7,
        width_ths=0.5,
        add_margin=0.1,
        x_ths=1.0,
        y_ths=0.5,
    )

    # Mock getattr for additional attributes that can't be set directly
    with patch("builtins.getattr") as mock_getattr:

        def getattr_side_effect(obj: Any, attr: str, default: Any = None) -> Any:
            advanced_attrs = {
                "batch_size": 8,
                "workers": 4,
                "allowlist": "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                "blocklist": "0123456789",
                "paragraph": True,
                "detail": 1,
            }
            if attr in advanced_attrs:
                return advanced_attrs[attr]
            try:
                return getattr(type(obj), attr, default)
            except AttributeError:
                return default

        mock_getattr.side_effect = getattr_side_effect

        result = process_image_easyocr_sync(sample_image_path, config)

        assert isinstance(result, ExtractionResult)
        assert result.content == "Test"

        # Verify readtext was called with correct parameters
        call_args = mock_easyocr.readtext.call_args
        assert call_args[0][0] == sample_image_path  # First positional arg is image path

        kwargs = call_args[1]
        assert kwargs["decoder"] == "beamsearch"
        assert kwargs["beamWidth"] == 10
        assert kwargs["batch_size"] == 8
        assert kwargs["workers"] == 4
        assert kwargs["allowlist"] == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        assert kwargs["blocklist"] == "0123456789"
        assert kwargs["detail"] == 1
        assert kwargs["rotation_info"] == [0, 90, 180, 270]
        assert kwargs["paragraph"] is True
        assert kwargs["min_size"] == 20
        assert kwargs["text_threshold"] == 0.7
        assert kwargs["low_text"] == 0.4
        assert kwargs["link_threshold"] == 0.4
        assert kwargs["canvas_size"] == 2560
        assert kwargs["mag_ratio"] == 1.0
        assert kwargs["slope_ths"] == 0.1
        assert kwargs["ycenter_ths"] == 0.5
        assert kwargs["height_ths"] == 0.7
        assert kwargs["width_ths"] == 0.5
        assert kwargs["add_margin"] == 0.1
        assert kwargs["x_ths"] == 1.0
        assert kwargs["y_ths"] == 0.5


def test_process_image_bytes_easyocr_sync_success(sample_image_bytes: bytes, mock_easyocr: Mock) -> None:
    """Test successful image bytes processing."""
    mock_easyocr.readtext.return_value = [([[0, 0], [100, 0], [100, 20], [0, 20]], "Bytes Test", 0.85)]

    config = EasyOCRConfig()
    result = process_image_bytes_easyocr_sync(sample_image_bytes, config)

    assert isinstance(result, ExtractionResult)
    assert result.content == "Bytes Test"
    assert result.metadata == {"confidence": 0.85}


def test_process_image_bytes_easyocr_sync_cleanup() -> None:
    """Test that temporary files are cleaned up after processing."""
    sample_bytes = b"fake image data"

    with patch("kreuzberg._ocr._sync.Image.open") as mock_open:
        mock_image = Mock()
        mock_open.return_value.__enter__.return_value = mock_image

        with patch("kreuzberg._ocr._sync.process_image_easyocr_sync") as mock_process:
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
                result = process_image_bytes_easyocr_sync(sample_bytes, None)

                assert isinstance(result, ExtractionResult)
                assert result.content == "test"

                # Verify temp files were cleaned up
                for temp_file in created_files:
                    assert not Path(temp_file).exists()


def test_process_batch_images_sync_success(mock_easyocr: Mock) -> None:
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
                    return [([[0, 0], [100, 0], [100, 20], [0, 20]], f"Image {i}", 0.9)]
            return [([[0, 0], [100, 0], [100, 20], [0, 20]], "Default", 0.9)]

        mock_easyocr.readtext.side_effect = side_effect

        config = EasyOCRConfig()
        results = process_batch_images_sync(cast("list[str | Path]", image_paths), config, backend="easyocr")

        assert len(results) == 3
        for i, result in enumerate(results):
            assert isinstance(result, ExtractionResult)
            assert result.content == f"Image {i}"
            assert result.metadata == {"confidence": 0.9}

    finally:
        # Cleanup temp files
        for path in image_paths:
            Path(path).unlink(missing_ok=True)


def test_process_batch_images_sync_empty_list(mock_easyocr: Mock) -> None:
    """Test batch processing with empty image list."""
    results = process_batch_images_sync([], None, backend="easyocr")
    assert results == []


def test_process_batch_images_threaded_success(mock_easyocr: Mock) -> None:
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
                    return [([[0, 0], [100, 0], [100, 20], [0, 20]], f"Threaded {i}", 0.8)]
            return [([[0, 0], [100, 0], [100, 20], [0, 20]], "Default", 0.8)]

        mock_easyocr.readtext.side_effect = side_effect

        config = EasyOCRConfig()
        results = process_batch_images_threaded(
            cast("list[str | Path]", image_paths), config, backend="easyocr", max_workers=2
        )

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


def test_process_batch_images_threaded_with_errors(mock_easyocr: Mock) -> None:
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
            return [([[0, 0], [100, 0], [100, 20], [0, 20]], "Success", 0.9)]

        mock_easyocr.readtext.side_effect = side_effect

        config = EasyOCRConfig()
        results = process_batch_images_threaded(
            cast("list[str | Path]", image_paths), config, backend="easyocr", max_workers=2
        )

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


def test_process_batch_images_threaded_default_workers(mock_easyocr: Mock) -> None:
    """Test threaded batch processing with default worker count."""
    image_paths = ["image1.png", "image2.png"]  # Fake paths for this test

    with patch("kreuzberg._ocr._sync.process_image_easyocr_sync") as mock_process:
        mock_process.return_value = ExtractionResult(
            content="test", mime_type=PLAIN_TEXT_MIME_TYPE, metadata={}, chunks=[]
        )

        # Patch multiprocessing.cpu_count inside the function
        with patch("multiprocessing.cpu_count", return_value=8):
            results = process_batch_images_threaded(
                cast("list[str | Path]", image_paths), None, backend="easyocr", max_workers=None
            )

            assert len(results) == 2
            # Should use min(len(image_paths), cpu_count()) = min(2, 8) = 2
            assert mock_process.call_count == 2


def test_process_batch_images_threaded_empty_list() -> None:
    """Test threaded batch processing with empty image list."""
    results = process_batch_images_threaded([], None, backend="tesseract", max_workers=2)
    assert results == []
