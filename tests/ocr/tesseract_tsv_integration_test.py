from __future__ import annotations

from pathlib import Path

import pytest
from PIL import Image

from kreuzberg._ocr._table_extractor import TesseractTableExtractor, extract_table_from_tsv
from kreuzberg._ocr._tesseract import TesseractBackend


@pytest.fixture
def table_image_path() -> Path:
    path = Path("tests/test_source_files/tables/simple_table.png")
    if not path.exists():
        pytest.skip(f"Test image not found: {path}")
    return path


@pytest.fixture
def science_table_image() -> Path:
    path = Path("tests/test_source_files/tables/complex_document.png")
    if not path.exists():
        pytest.skip(f"Test image not found: {path}")
    return path


@pytest.mark.anyio
async def test_tesseract_tsv_output_integration(table_image_path: Path) -> None:
    backend = TesseractBackend()

    result = await backend.process_file(table_image_path, output_format="tsv", enable_table_detection=False)

    assert result is not None
    assert isinstance(result.content, str)
    assert len(result.content) > 0


@pytest.mark.anyio
async def test_tesseract_process_image_with_table_detection(table_image_path: Path) -> None:
    backend = TesseractBackend()

    with Image.open(table_image_path) as img:
        result = await backend.process_image(
            img,
            enable_table_detection=True,
            table_column_threshold=30,
            table_row_threshold_ratio=0.5,
        )

    assert result is not None
    assert len(result.content) > 0

    if result.tables:
        for _i, _table in enumerate(result.tables):
            pass


@pytest.mark.anyio
async def test_table_detection_enabled(table_image_path: Path) -> None:
    backend = TesseractBackend()

    result = await backend.process_file(
        table_image_path,
        enable_table_detection=True,
        table_column_threshold=20,
        table_row_threshold_ratio=0.5,
    )

    assert result is not None
    if result.tables:
        table = result.tables[0]
        assert "text" in table
        assert "|" in table["text"]
    else:
        assert len(result.content) > 0


def test_table_extractor_with_real_tsv() -> None:
    tsv_data = """level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
1\t1\t0\t0\t0\t0\t0\t0\t800\t600\t-1\t
5\t1\t1\t1\t1\t1\t100\t100\t80\t30\t95.0\tProduct
5\t1\t1\t1\t1\t2\t250\t100\t60\t30\t94.0\tPrice
5\t1\t1\t1\t1\t3\t400\t100\t80\t30\t96.0\tQuantity
5\t1\t2\t1\t1\t1\t100\t150\t80\t30\t92.0\tApples
5\t1\t2\t1\t1\t2\t250\t150\t60\t30\t93.0\t$2.50
5\t1\t2\t1\t1\t3\t400\t150\t40\t30\t91.0\t10
5\t1\t3\t1\t1\t1\t100\t200\t80\t30\t94.0\tBananas
5\t1\t3\t1\t1\t2\t250\t200\t60\t30\t92.0\t$1.20
5\t1\t3\t1\t1\t3\t400\t200\t40\t30\t93.0\t15"""

    extractor = TesseractTableExtractor()
    words = extractor.extract_words(tsv_data)

    assert len(words) == 9
    assert words[0]["text"] == "Product"

    cols = extractor.detect_columns(words)
    assert len(cols) == 3
    assert 90 < cols[0] < 110
    assert 240 < cols[1] < 260
    assert 390 < cols[2] < 410

    rows = extractor.detect_rows(words)
    assert len(rows) == 3

    table = extractor.reconstruct_table(words)
    assert len(table) == 3
    assert table[0] == ["Product", "Price", "Quantity"]
    assert table[1] == ["Apples", "$2.50", "10"]
    assert table[2] == ["Bananas", "$1.20", "15"]

    markdown = extractor.to_markdown(table)
    assert "| Product | Price | Quantity |" in markdown
    assert "| --- | --- | --- |" in markdown
    assert "| Apples | $2.50 | 10 |" in markdown


def test_extract_table_from_tsv_convenience() -> None:
    tsv_data = """level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t50\t50\t40\t20\t95.0\tA
5\t1\t1\t1\t1\t2\t150\t50\t40\t20\t94.0\tB
5\t1\t2\t1\t1\t1\t50\t100\t40\t20\t93.0\t1
5\t1\t2\t1\t1\t2\t150\t100\t40\t20\t92.0\t2"""

    markdown = extract_table_from_tsv(tsv_data)

    assert markdown != ""
    assert "| A | B |" in markdown
    assert "| 1 | 2 |" in markdown


def test_table_extraction_with_empty_cells() -> None:
    tsv_data = """level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t50\t50\t60\t30\t95.0\tHeader1
5\t1\t1\t1\t1\t2\t200\t50\t60\t30\t94.0\tHeader2
5\t1\t1\t1\t1\t3\t350\t50\t60\t30\t96.0\tHeader3
5\t1\t2\t1\t1\t1\t50\t100\t60\t30\t92.0\tData1
5\t1\t2\t1\t1\t2\t350\t100\t60\t30\t91.0\tData3"""

    extractor = TesseractTableExtractor()
    words = extractor.extract_words(tsv_data)
    table = extractor.reconstruct_table(words)

    assert len(table) == 2
    assert table[0] == ["Header1", "Header2", "Header3"]
    assert table[1][0] == "Data1"
    assert table[1][1] == ""
    assert table[1][2] == "Data3"


def test_table_extraction_confidence_threshold() -> None:
    tsv_data = """level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t50\t50\t60\t30\t95.0\tGood
5\t1\t1\t1\t1\t2\t150\t50\t60\t30\t20.0\tBad
5\t1\t1\t1\t1\t3\t250\t50\t60\t30\t85.0\tAlsoGood"""

    extractor = TesseractTableExtractor(min_confidence=30.0)
    words = extractor.extract_words(tsv_data)

    assert len(words) == 2
    assert words[0]["text"] == "Good"
    assert words[1]["text"] == "AlsoGood"


@pytest.mark.parametrize(
    "column_threshold,expected_cols",
    [
        (10, 3),
        (50, 2),
        (200, 1),
    ],
)
def test_column_clustering_thresholds(column_threshold: int, expected_cols: int) -> None:
    tsv_data = """level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t50\t50\t40\t30\t95.0\tA
5\t1\t1\t1\t1\t2\t80\t50\t40\t30\t94.0\tB
5\t1\t1\t1\t1\t3\t200\t50\t40\t30\t93.0\tC"""

    extractor = TesseractTableExtractor(column_threshold=column_threshold)
    words = extractor.extract_words(tsv_data)
    cols = extractor.detect_columns(words)

    assert len(cols) == expected_cols
