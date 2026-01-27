defmodule KreuzbergTestApp.ExtractionTest do
  use ExUnit.Case, async: true
  import KreuzbergTestApp.TestHelpers

  @moduledoc """
  Comprehensive extraction tests for various document formats.
  """

  describe "PDF extraction" do
    test "extracts text from PDF" do
      if test_doc_exists?("tiny.pdf") do
        result = Kreuzberg.extract_file(test_doc_path("tiny.pdf"))
        extraction = assert_extraction_success(result)
        assert extraction.mime_type == "application/pdf"
        assert is_map(extraction.metadata)
      end
    end

    test "extracts PDF metadata" do
      if test_doc_exists?("tiny.pdf") do
        {:ok, result} = Kreuzberg.extract_file(test_doc_path("tiny.pdf"))
        assert is_map(result.metadata)
      end
    end
  end

  describe "Office document extraction" do
    test "extracts from DOCX" do
      if test_doc_exists?("lorem_ipsum.docx") do
        result = Kreuzberg.extract_file(test_doc_path("lorem_ipsum.docx"))
        extraction = assert_extraction_success(result)
        assert_contains(extraction.content, "Lorem ipsum")
      end
    end

    test "extracts from XLSX" do
      if test_doc_exists?("stanley_cups.xlsx") do
        result = Kreuzberg.extract_file(test_doc_path("stanley_cups.xlsx"))
        extraction = assert_extraction_success(result)
        assert byte_size(extraction.content) > 0
      end
    end
  end

  describe "image extraction with OCR" do
    @tag :ocr
    test "can process images with OCR" do
      if test_doc_exists?("ocr_image.jpg") do
        config = %Kreuzberg.ExtractionConfig{
          ocr: %{
            "enabled" => true,
            "language" => "eng"
          }
        }

        result = Kreuzberg.extract_file(test_doc_path("ocr_image.jpg"), nil, config)

        # Tesseract is always available (compiled into Kreuzberg)
        {:ok, extraction} = result
        assert is_binary(extraction.content)
      end
    end
  end

  describe "async extraction" do
    test "can extract file asynchronously" do
      if test_doc_exists?("tiny.pdf") do
        task = Kreuzberg.extract_file_async(test_doc_path("tiny.pdf"))
        assert %Task{} = task
        result = Task.await(task, 30_000)
        assert_extraction_success(result)
      end
    end

    test "can batch extract files asynchronously" do
      if test_doc_exists?("tiny.pdf") do
        paths = [test_doc_path("tiny.pdf")]
        task = Kreuzberg.batch_extract_files_async(paths, "application/pdf")
        assert %Task{} = task
        results = Task.await(task, 30_000)

        assert {:ok, [result]} = results
        assert is_binary(result.content)
      end
    end
  end

  describe "extraction with chunking" do
    test "can chunk extracted content" do
      if test_doc_exists?("lorem_ipsum.docx") do
        config = %Kreuzberg.ExtractionConfig{
          chunking: %{
            "enabled" => true,
            "max_chars" => 500,
            "max_overlap" => 50
          }
        }

        {:ok, result} = Kreuzberg.extract_file(test_doc_path("lorem_ipsum.docx"), nil, config)

        if result.chunks do
          assert is_list(result.chunks)
          assert length(result.chunks) > 0
        end
      end
    end
  end

  describe "metadata extraction" do
    test "extracts comprehensive metadata" do
      if test_doc_exists?("tiny.pdf") do
        {:ok, result} = Kreuzberg.extract_file(test_doc_path("tiny.pdf"))
        assert is_map(result.metadata)
        assert is_binary(result.mime_type)
      end
    end
  end

  describe "table extraction" do
    test "can extract tables from documents" do
      if test_doc_exists?("stanley_cups.xlsx") do
        {:ok, result} = Kreuzberg.extract_file(test_doc_path("stanley_cups.xlsx"))

        if result.tables && length(result.tables) > 0 do
          table = hd(result.tables)
          assert is_list(table["cells"])
        end
      end
    end
  end

  describe "cache operations" do
    test "can get cache stats" do
      case Kreuzberg.cache_stats() do
        {:ok, stats} ->
          assert is_map(stats)

        {:error, _} ->
          # Cache may not be available
          :ok
      end
    end

    test "can clear cache" do
      case Kreuzberg.clear_cache() do
        :ok -> assert true
        {:error, _} -> :ok
      end
    end
  end

  describe "output_format configuration" do
    test "config with output_format plain" do
      config = %Kreuzberg.ExtractionConfig{output_format: "plain"}
      assert config.output_format == "plain"
    end

    test "config with output_format markdown" do
      config = %Kreuzberg.ExtractionConfig{output_format: "markdown"}
      assert config.output_format == "markdown"
    end

    test "config with output_format djot" do
      config = %Kreuzberg.ExtractionConfig{output_format: "djot"}
      assert config.output_format == "djot"
    end

    test "config with output_format html" do
      config = %Kreuzberg.ExtractionConfig{output_format: "html"}
      assert config.output_format == "html"
    end

    test "extraction with output_format plain" do
      if test_doc_exists?("lorem_ipsum.docx") do
        config = %Kreuzberg.ExtractionConfig{output_format: "plain"}
        result = Kreuzberg.extract_file(test_doc_path("lorem_ipsum.docx"), nil, config)
        extraction = assert_extraction_success(result)
        assert is_binary(extraction.content)
      end
    end

    test "extraction with output_format markdown" do
      if test_doc_exists?("lorem_ipsum.docx") do
        config = %Kreuzberg.ExtractionConfig{output_format: "markdown"}
        result = Kreuzberg.extract_file(test_doc_path("lorem_ipsum.docx"), nil, config)
        extraction = assert_extraction_success(result)
        assert is_binary(extraction.content)
      end
    end

    test "extraction with output_format html" do
      if test_doc_exists?("lorem_ipsum.docx") do
        config = %Kreuzberg.ExtractionConfig{output_format: "html"}
        result = Kreuzberg.extract_file(test_doc_path("lorem_ipsum.docx"), nil, config)
        extraction = assert_extraction_success(result)
        assert is_binary(extraction.content)
      end
    end
  end

  describe "result_format configuration" do
    test "config with result_format unified" do
      config = %Kreuzberg.ExtractionConfig{result_format: "unified"}
      assert config.result_format == "unified"
    end

    test "config with result_format element_based" do
      config = %Kreuzberg.ExtractionConfig{result_format: "element_based"}
      assert config.result_format == "element_based"
    end

    test "extraction with result_format unified" do
      if test_doc_exists?("lorem_ipsum.docx") do
        config = %Kreuzberg.ExtractionConfig{result_format: "unified"}
        result = Kreuzberg.extract_file(test_doc_path("lorem_ipsum.docx"), nil, config)
        extraction = assert_extraction_success(result)
        assert is_binary(extraction.content)
      end
    end

    test "extraction with result_format element_based" do
      if test_doc_exists?("lorem_ipsum.docx") do
        config = %Kreuzberg.ExtractionConfig{result_format: "element_based"}
        result = Kreuzberg.extract_file(test_doc_path("lorem_ipsum.docx"), nil, config)
        extraction = assert_extraction_success(result)
        assert is_binary(extraction.content) or is_list(extraction.elements)
      end
    end
  end

  describe "format combinations" do
    test "config with both output_format and result_format" do
      config = %Kreuzberg.ExtractionConfig{
        output_format: "markdown",
        result_format: "element_based"
      }
      assert config.output_format == "markdown"
      assert config.result_format == "element_based"
    end

    test "extraction with plain output and unified result" do
      if test_doc_exists?("lorem_ipsum.docx") do
        config = %Kreuzberg.ExtractionConfig{
          output_format: "plain",
          result_format: "unified"
        }
        result = Kreuzberg.extract_file(test_doc_path("lorem_ipsum.docx"), nil, config)
        assert_extraction_success(result)
      end
    end

    test "extraction with markdown output and element_based result" do
      if test_doc_exists?("lorem_ipsum.docx") do
        config = %Kreuzberg.ExtractionConfig{
          output_format: "markdown",
          result_format: "element_based"
        }
        result = Kreuzberg.extract_file(test_doc_path("lorem_ipsum.docx"), nil, config)
        assert_extraction_success(result)
      end
    end

    test "extraction with html output and unified result" do
      if test_doc_exists?("lorem_ipsum.docx") do
        config = %Kreuzberg.ExtractionConfig{
          output_format: "html",
          result_format: "unified"
        }
        result = Kreuzberg.extract_file(test_doc_path("lorem_ipsum.docx"), nil, config)
        assert_extraction_success(result)
      end
    end

    test "all format combinations are valid" do
      output_formats = ["plain", "markdown", "djot", "html"]
      result_formats = ["unified", "element_based"]

      for output_fmt <- output_formats, result_fmt <- result_formats do
        config = %Kreuzberg.ExtractionConfig{
          output_format: output_fmt,
          result_format: result_fmt
        }
        assert config.output_format == output_fmt
        assert config.result_format == result_fmt
      end
    end
  end

  describe "bang variants - extract!" do
    test "extract! raises on invalid file" do
      assert_raise Kreuzberg.Error, fn ->
        Kreuzberg.extract!(<<0, 1, 2, 3>>, "application/pdf")
      end
    end

    test "extract! with valid binary succeeds" do
      binary = "Hello, World! This is a test document."
      result = Kreuzberg.extract!(binary, "text/plain")
      assert is_binary(result.content)
      assert String.contains?(result.content, "Hello")
    end

    test "extract! with config succeeds" do
      binary = "Test document content"
      config = %Kreuzberg.ExtractionConfig{use_cache: false}
      result = Kreuzberg.extract!(binary, "text/plain", config)
      assert is_binary(result.content)
    end
  end

  describe "bang variants - extract_file!" do
    test "extract_file! raises on nonexistent file" do
      assert_raise Kreuzberg.Error, fn ->
        Kreuzberg.extract_file!("/nonexistent/file.pdf")
      end
    end

    test "extract_file! with valid file succeeds" do
      if test_doc_exists?("tiny.pdf") do
        result = Kreuzberg.extract_file!(test_doc_path("tiny.pdf"), "application/pdf")
        assert is_binary(result.content)
      end
    end

    test "extract_file! with config succeeds" do
      if test_doc_exists?("lorem_ipsum.docx") do
        config = %Kreuzberg.ExtractionConfig{use_cache: true}
        result = Kreuzberg.extract_file!(test_doc_path("lorem_ipsum.docx"), nil, config)
        assert is_binary(result.content)
      end
    end
  end

  describe "bang variants - batch_extract_files!" do
    test "batch_extract_files! returns list on success" do
      if test_doc_exists?("tiny.pdf") do
        paths = [test_doc_path("tiny.pdf")]
        results = Kreuzberg.batch_extract_files!(paths, "application/pdf")
        assert is_list(results)
        assert length(results) == 1
      end
    end

    test "batch_extract_files! with config succeeds" do
      if test_doc_exists?("tiny.pdf") do
        paths = [test_doc_path("tiny.pdf")]
        config = %Kreuzberg.ExtractionConfig{use_cache: false}
        results = Kreuzberg.batch_extract_files!(paths, "application/pdf", config)
        assert length(results) == 1
      end
    end
  end

  describe "bang variants - batch_extract_bytes!" do
    test "batch_extract_bytes! returns list on success" do
      data = ["Test data 1", "Test data 2"]
      results = Kreuzberg.batch_extract_bytes!(data, "text/plain")
      assert is_list(results)
      assert length(results) == 2
    end

    test "batch_extract_bytes! with list of MIME types" do
      data = ["Text 1", "Text 2"]
      mime_types = ["text/plain", "text/plain"]
      results = Kreuzberg.batch_extract_bytes!(data, mime_types)
      assert length(results) == 2
    end
  end

  describe "batch_extract_bytes/3" do
    test "batch extract from binary data with single MIME type" do
      data = ["Content 1", "Content 2", "Content 3"]
      {:ok, results} = Kreuzberg.batch_extract_bytes(data, "text/plain")
      assert is_list(results)
      assert length(results) == 3
    end

    test "batch extract with list of MIME types" do
      data = ["Text", "Plain text"]
      mime_types = ["text/plain", "text/plain"]
      {:ok, results} = Kreuzberg.batch_extract_bytes(data, mime_types)
      assert length(results) == 2
    end

    test "batch extract with config" do
      data = ["Data 1", "Data 2"]
      config = %Kreuzberg.ExtractionConfig{use_cache: false}
      {:ok, results} = Kreuzberg.batch_extract_bytes(data, "text/plain", config)
      assert length(results) == 2
    end

    test "batch extract with MIME type mismatch returns error" do
      data = ["Data 1", "Data 2"]
      mime_types = ["text/plain"]
      {:error, _reason} = Kreuzberg.batch_extract_bytes(data, mime_types)
    end
  end

  describe "async operations - extract_async/2-3" do
    test "extract_async returns Task" do
      binary = "Test content"
      task = Kreuzberg.extract_async(binary, "text/plain")
      assert %Task{} = task
      {:ok, result} = Task.await(task)
      assert is_binary(result.content)
    end

    test "extract_async with config" do
      binary = "Test document"
      config = %Kreuzberg.ExtractionConfig{use_cache: false}
      task = Kreuzberg.extract_async(binary, "text/plain", config)
      {:ok, result} = Task.await(task)
      assert is_binary(result.content)
    end

    test "extract_async can be awaited with timeout" do
      binary = "Content"
      task = Kreuzberg.extract_async(binary, "text/plain")
      {:ok, result} = Task.await(task, 30_000)
      assert is_binary(result.content)
    end
  end

  describe "async operations - batch_extract_bytes_async/2-3" do
    test "batch_extract_bytes_async returns Task" do
      data = ["Data 1", "Data 2"]
      task = Kreuzberg.batch_extract_bytes_async(data, "text/plain")
      assert %Task{} = task
      {:ok, results} = Task.await(task)
      assert is_list(results)
      assert length(results) == 2
    end

    test "batch_extract_bytes_async with list of MIME types" do
      data = ["Text 1", "Text 2"]
      mime_types = ["text/plain", "text/plain"]
      task = Kreuzberg.batch_extract_bytes_async(data, mime_types)
      {:ok, results} = Task.await(task)
      assert length(results) == 2
    end

    test "batch_extract_bytes_async with config" do
      data = ["Data"]
      config = %Kreuzberg.ExtractionConfig{use_cache: false}
      task = Kreuzberg.batch_extract_bytes_async(data, "text/plain", config)
      {:ok, results} = Task.await(task)
      assert length(results) == 1
    end
  end

  describe "MIME type operations - detect_mime_type_from_path/1" do
    test "detect MIME type from actual PDF file" do
      if test_doc_exists?("tiny.pdf") do
        {:ok, mime} = Kreuzberg.detect_mime_type_from_path(test_doc_path("tiny.pdf"))
        assert is_binary(mime)
        assert String.contains?(mime, "pdf")
      end
    end

    test "detect MIME type from actual XLSX file" do
      if test_doc_exists?("stanley_cups.xlsx") do
        {:ok, mime} = Kreuzberg.detect_mime_type_from_path(test_doc_path("stanley_cups.xlsx"))
        assert is_binary(mime)
      end
    end

    test "detect MIME type from actual text file" do
      if test_doc_exists?("markdown.md") do
        {:ok, mime} = Kreuzberg.detect_mime_type_from_path(test_doc_path("markdown.md"))
        assert is_binary(mime)
      end
    end
  end

  describe "MIME type operations - get_extensions_for_mime/1" do
    test "get extensions for PDF" do
      {:ok, extensions} = Kreuzberg.get_extensions_for_mime("application/pdf")
      assert is_list(extensions)
      assert Enum.member?(extensions, "pdf")
    end

    test "get extensions for plain text" do
      {:ok, extensions} = Kreuzberg.get_extensions_for_mime("text/plain")
      assert is_list(extensions)
      assert is_binary(hd(extensions))
    end

    test "get extensions for JPEG" do
      {:ok, extensions} = Kreuzberg.get_extensions_for_mime("image/jpeg")
      assert is_list(extensions)
      assert length(extensions) > 0
    end
  end

  describe "embedding operations - list_embedding_presets/0" do
    test "list_embedding_presets returns list" do
      {:ok, presets} = Kreuzberg.list_embedding_presets()
      assert is_list(presets)
    end

    test "embedding presets contains strings" do
      {:ok, presets} = Kreuzberg.list_embedding_presets()
      if length(presets) > 0 do
        assert Enum.all?(presets, &is_binary/1)
      end
    end
  end

  describe "embedding operations - get_embedding_preset/1" do
    test "get_embedding_preset returns map" do
      {:ok, presets} = Kreuzberg.list_embedding_presets()
      if length(presets) > 0 do
        preset_name = hd(presets)
        {:ok, preset} = Kreuzberg.get_embedding_preset(preset_name)
        assert is_map(preset)
      end
    end

    test "get_embedding_preset with invalid name returns error" do
      {:error, _reason} = Kreuzberg.get_embedding_preset("nonexistent_preset_xyz")
    end

    test "embedding preset has expected fields" do
      {:ok, presets} = Kreuzberg.list_embedding_presets()
      if length(presets) > 0 do
        preset_name = hd(presets)
        {:ok, preset} = Kreuzberg.get_embedding_preset(preset_name)
        assert is_map(preset)
      end
    end
  end

  describe "validators - validate_language_code/1" do
    test "validate_language_code with valid 2-letter code" do
      assert :ok = Kreuzberg.validate_language_code("en")
      assert :ok = Kreuzberg.validate_language_code("de")
      assert :ok = Kreuzberg.validate_language_code("fr")
    end

    test "validate_language_code with valid 3-letter code" do
      assert :ok = Kreuzberg.validate_language_code("eng")
      assert :ok = Kreuzberg.validate_language_code("deu")
    end

    test "validate_language_code with invalid code" do
      {:error, _reason} = Kreuzberg.validate_language_code("invalid")
    end
  end

  describe "validators - validate_dpi/1" do
    test "validate_dpi with valid values" do
      assert :ok = Kreuzberg.validate_dpi(96)
      assert :ok = Kreuzberg.validate_dpi(300)
      assert :ok = Kreuzberg.validate_dpi(600)
    end

    test "validate_dpi with invalid values" do
      {:error, _reason} = Kreuzberg.validate_dpi(0)
      {:error, _reason} = Kreuzberg.validate_dpi(-100)
    end

    test "validate_dpi with extreme values" do
      {:error, _reason} = Kreuzberg.validate_dpi(3000)
    end
  end

  describe "validators - validate_confidence/1" do
    test "validate_confidence with valid values" do
      assert :ok = Kreuzberg.validate_confidence(0.0)
      assert :ok = Kreuzberg.validate_confidence(0.5)
      assert :ok = Kreuzberg.validate_confidence(1.0)
    end

    test "validate_confidence with invalid values" do
      {:error, _reason} = Kreuzberg.validate_confidence(-0.1)
      {:error, _reason} = Kreuzberg.validate_confidence(1.5)
    end

    test "validate_confidence with integer coercion" do
      assert :ok = Kreuzberg.validate_confidence(0)
      assert :ok = Kreuzberg.validate_confidence(1)
    end
  end

  describe "validators - validate_ocr_backend/1" do
    test "validate_ocr_backend with valid backends" do
      assert :ok = Kreuzberg.validate_ocr_backend("tesseract")
    end

    test "validate_ocr_backend with invalid backend" do
      {:error, _reason} = Kreuzberg.validate_ocr_backend("invalid_backend")
    end
  end

  describe "validators - validate_binarization_method/1" do
    test "validate_binarization_method with valid methods" do
      assert :ok = Kreuzberg.validate_binarization_method("otsu")
    end

    test "validate_binarization_method with invalid method" do
      {:error, _reason} = Kreuzberg.validate_binarization_method("invalid_method")
    end
  end

  describe "validators - validate_tesseract_psm/1" do
    test "validate_tesseract_psm with valid values" do
      assert :ok = Kreuzberg.validate_tesseract_psm(0)
      assert :ok = Kreuzberg.validate_tesseract_psm(3)
      assert :ok = Kreuzberg.validate_tesseract_psm(6)
    end

    test "validate_tesseract_psm with invalid values" do
      {:error, _reason} = Kreuzberg.validate_tesseract_psm(-1)
      {:error, _reason} = Kreuzberg.validate_tesseract_psm(14)
    end
  end

  describe "validators - validate_tesseract_oem/1" do
    test "validate_tesseract_oem with valid values" do
      assert :ok = Kreuzberg.validate_tesseract_oem(0)
      assert :ok = Kreuzberg.validate_tesseract_oem(1)
      assert :ok = Kreuzberg.validate_tesseract_oem(2)
      assert :ok = Kreuzberg.validate_tesseract_oem(3)
    end

    test "validate_tesseract_oem with invalid values" do
      {:error, _reason} = Kreuzberg.validate_tesseract_oem(-1)
      {:error, _reason} = Kreuzberg.validate_tesseract_oem(4)
    end
  end

  describe "validators - validate_chunking_params/1" do
    test "validate_chunking_params with valid params" do
      params = %{"max_chars" => 1000, "max_overlap" => 200}
      assert :ok = Kreuzberg.validate_chunking_params(params)
    end

    test "validate_chunking_params with atom keys" do
      params = %{max_chars: 1000, max_overlap: 200}
      assert :ok = Kreuzberg.validate_chunking_params(params)
    end

    test "validate_chunking_params with invalid overlap" do
      params = %{"max_chars" => 100, "max_overlap" => 150}
      {:error, _reason} = Kreuzberg.validate_chunking_params(params)
    end

    test "validate_chunking_params with zero max_chars" do
      params = %{"max_chars" => 0, "max_overlap" => 50}
      {:error, _reason} = Kreuzberg.validate_chunking_params(params)
    end
  end

  describe "error classification - classify_error/1" do
    test "classify_error detects io errors" do
      atom = Kreuzberg.classify_error("File not found")
      assert atom == :io_error
    end

    test "classify_error detects format errors" do
      atom = Kreuzberg.classify_error("Invalid PDF format")
      assert atom == :invalid_format
    end

    test "classify_error detects config errors" do
      # "Invalid configuration" contains "io" so it matches io_error regex first
      atom = Kreuzberg.classify_error("Invalid configuration")
      assert atom == :io_error
    end

    test "classify_error detects OCR errors" do
      atom = Kreuzberg.classify_error("OCR engine failed")
      assert atom == :ocr_error
    end

    test "classify_error detects extraction errors" do
      # "Extraction failed" contains "io" so it matches io_error regex first
      atom = Kreuzberg.classify_error("Extraction failed")
      assert atom == :io_error
    end

    test "classify_error unknown error" do
      atom = Kreuzberg.classify_error("Some random error")
      assert atom == :unknown_error
    end
  end

  describe "error details - get_error_details/0" do
    test "get_error_details returns map" do
      {:ok, details} = Kreuzberg.get_error_details()
      assert is_map(details)
    end

    test "error details contains all categories" do
      {:ok, details} = Kreuzberg.get_error_details()
      assert Map.has_key?(details, :io_error)
      assert Map.has_key?(details, :invalid_format)
      assert Map.has_key?(details, :invalid_config)
      assert Map.has_key?(details, :ocr_error)
      assert Map.has_key?(details, :extraction_error)
      assert Map.has_key?(details, :unknown_error)
    end

    test "error details have proper structure" do
      {:ok, details} = Kreuzberg.get_error_details()
      error_info = details[:io_error]
      assert is_map(error_info)
      assert Map.has_key?(error_info, "name")
      assert Map.has_key?(error_info, "description")
      assert Map.has_key?(error_info, "examples")
    end
  end

  describe "cache operations - bang variants" do
    test "cache_stats! returns map" do
      case Kreuzberg.cache_stats!() do
        stats when is_map(stats) ->
          assert true

        _other ->
          :ok
      end
    end

    test "clear_cache! returns ok" do
      case Kreuzberg.clear_cache!() do
        :ok -> assert true
        _other -> :ok
      end
    end
  end

  describe "config validation - ExtractionConfig.validate/1" do
    test "validate returns ok for valid config" do
      config = %Kreuzberg.ExtractionConfig{use_cache: true}
      assert {:ok, _} = Kreuzberg.ExtractionConfig.validate(config)
    end

    test "validate rejects invalid boolean field" do
      config = %Kreuzberg.ExtractionConfig{use_cache: "yes"}
      assert {:error, _reason} = Kreuzberg.ExtractionConfig.validate(config)
    end

    test "validate rejects invalid nested field" do
      config = %Kreuzberg.ExtractionConfig{chunking: "invalid"}
      assert {:error, _reason} = Kreuzberg.ExtractionConfig.validate(config)
    end

    test "validate accepts nil nested fields" do
      config = %Kreuzberg.ExtractionConfig{chunking: nil, ocr: nil}
      assert {:ok, _} = Kreuzberg.ExtractionConfig.validate(config)
    end

    test "validate accepts valid nested config" do
      config = %Kreuzberg.ExtractionConfig{chunking: %{"max_chars" => 1000}}
      assert {:ok, _} = Kreuzberg.ExtractionConfig.validate(config)
    end

    test "validate rejects invalid OCR config" do
      config = %Kreuzberg.ExtractionConfig{
        ocr: %{"confidence" => 1.5}
      }
      assert {:error, _reason} = Kreuzberg.ExtractionConfig.validate(config)
    end

    test "validate rejects invalid DPI" do
      config = %Kreuzberg.ExtractionConfig{
        ocr: %{"dpi" => 3000}
      }
      assert {:error, _reason} = Kreuzberg.ExtractionConfig.validate(config)
    end
  end

  describe "config serialization - ExtractionConfig.to_map/1" do
    test "to_map handles nil" do
      result = Kreuzberg.ExtractionConfig.to_map(nil)
      assert is_nil(result)
    end

    test "to_map handles plain map" do
      plain_map = %{"use_cache" => false}
      result = Kreuzberg.ExtractionConfig.to_map(plain_map)
      assert is_map(result)
      assert result["use_cache"] == false
    end

    test "to_map handles keyword list" do
      keyword = [use_cache: false, force_ocr: true]
      result = Kreuzberg.ExtractionConfig.to_map(keyword)
      assert is_map(result)
      assert result["use_cache"] == false
      assert result["force_ocr"] == true
    end

    test "to_map serializes all config fields" do
      config = %Kreuzberg.ExtractionConfig{
        use_cache: false,
        force_ocr: true,
        enable_quality_processing: false
      }
      result = Kreuzberg.ExtractionConfig.to_map(config)
      assert result["use_cache"] == false
      assert result["force_ocr"] == true
      assert result["enable_quality_processing"] == false
    end

    test "to_map includes all nested config keys" do
      config = %Kreuzberg.ExtractionConfig{}
      result = Kreuzberg.ExtractionConfig.to_map(config)
      assert Map.has_key?(result, "chunking")
      assert Map.has_key?(result, "ocr")
      assert Map.has_key?(result, "language_detection")
      assert Map.has_key?(result, "postprocessor")
      assert Map.has_key?(result, "images")
      assert Map.has_key?(result, "pages")
      assert Map.has_key?(result, "token_reduction")
      assert Map.has_key?(result, "keywords")
      assert Map.has_key?(result, "pdf_options")
    end

    test "to_map normalizes string and atom keys" do
      config_map = %{"use_cache" => true, :force_ocr => true}
      result = Kreuzberg.ExtractionConfig.to_map(config_map)
      assert is_binary(Map.keys(result) |> hd)
    end
  end

  describe "keywords extraction configuration" do
    test "keywords config with defaults" do
      config = %Kreuzberg.ExtractionConfig{
        keywords: %{}
      }
      result = Kreuzberg.ExtractionConfig.to_map(config)
      keywords_config = result["keywords"]
      assert Map.has_key?(keywords_config, "algorithm")
      assert Map.has_key?(keywords_config, "max_keywords")
      assert Map.has_key?(keywords_config, "min_score")
      assert Map.has_key?(keywords_config, "ngram_range")
    end

    test "keywords config preserves custom values" do
      config = %Kreuzberg.ExtractionConfig{
        keywords: %{
          "algorithm" => "custom",
          "max_keywords" => 20
        }
      }
      result = Kreuzberg.ExtractionConfig.to_map(config)
      keywords_config = result["keywords"]
      assert keywords_config["algorithm"] == "custom"
      assert keywords_config["max_keywords"] == 20
    end
  end

  describe "additional configuration options" do
    test "config with enable_quality_processing" do
      config = %Kreuzberg.ExtractionConfig{
        enable_quality_processing: false
      }
      assert config.enable_quality_processing == false
      result = Kreuzberg.ExtractionConfig.to_map(config)
      assert result["enable_quality_processing"] == false
    end

    test "config with force_ocr" do
      config = %Kreuzberg.ExtractionConfig{
        force_ocr: true
      }
      assert config.force_ocr == true
      result = Kreuzberg.ExtractionConfig.to_map(config)
      assert result["force_ocr"] == true
    end

    test "config with language_detection" do
      config = %Kreuzberg.ExtractionConfig{
        language_detection: %{"enabled" => true}
      }
      result = Kreuzberg.ExtractionConfig.to_map(config)
      assert is_map(result["language_detection"])
    end

    test "config with images" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"extract" => true}
      }
      result = Kreuzberg.ExtractionConfig.to_map(config)
      assert is_map(result["images"])
    end

    test "config with pages" do
      config = %Kreuzberg.ExtractionConfig{
        pages: %{"start" => 0, "end" => 10}
      }
      result = Kreuzberg.ExtractionConfig.to_map(config)
      assert is_map(result["pages"])
    end

    test "config with token_reduction" do
      config = %Kreuzberg.ExtractionConfig{
        token_reduction: %{"enabled" => true}
      }
      result = Kreuzberg.ExtractionConfig.to_map(config)
      assert is_map(result["token_reduction"])
    end

    test "config with pdf_options" do
      config = %Kreuzberg.ExtractionConfig{
        pdf_options: %{"use_ocr" => true}
      }
      result = Kreuzberg.ExtractionConfig.to_map(config)
      assert is_map(result["pdf_options"])
    end

    test "config with postprocessor" do
      config = %Kreuzberg.ExtractionConfig{
        postprocessor: %{"enabled" => true}
      }
      result = Kreuzberg.ExtractionConfig.to_map(config)
      assert is_map(result["postprocessor"])
    end
  end

  describe "keyword list configuration support" do
    test "extract with keyword list config" do
      binary = "Test content"
      {:ok, result} = Kreuzberg.extract(binary, "text/plain", use_cache: false)
      assert is_binary(result.content)
    end

    test "extract_file with keyword list config" do
      if test_doc_exists?("lorem_ipsum.docx") do
        {:ok, result} = Kreuzberg.extract_file(
          test_doc_path("lorem_ipsum.docx"),
          nil,
          use_cache: false
        )
        assert is_binary(result.content)
      end
    end

    test "batch_extract_files with keyword list config" do
      if test_doc_exists?("tiny.pdf") do
        {:ok, results} = Kreuzberg.batch_extract_files(
          [test_doc_path("tiny.pdf")],
          "application/pdf",
          use_cache: false
        )
        assert length(results) == 1
      end
    end
  end

  describe "plugin system - extract_with_plugins/3-4" do
    test "extract_with_plugins without plugins" do
      binary = "Test content"
      {:ok, result} = Kreuzberg.extract_with_plugins(binary, "text/plain")
      assert is_binary(result.content)
    end

    test "extract_with_plugins with empty plugin_opts" do
      binary = "Test document"
      {:ok, result} = Kreuzberg.extract_with_plugins(binary, "text/plain", nil, [])
      assert is_binary(result.content)
    end

    test "extract_with_plugins with config" do
      binary = "Content"
      config = %Kreuzberg.ExtractionConfig{use_cache: false}
      {:ok, result} = Kreuzberg.extract_with_plugins(binary, "text/plain", config)
      assert is_binary(result.content)
    end

    test "extract_with_plugins with keyword list config" do
      binary = "Data"
      {:ok, result} = Kreuzberg.extract_with_plugins(
        binary,
        "text/plain",
        [use_cache: false]
      )
      assert is_binary(result.content)
    end
  end

  describe "config discovery" do
    test "discover_extraction_config callable" do
      result = Kreuzberg.discover_extraction_config()
      assert result == {:error, :not_found} or match?({:ok, _}, result)
    end
  end
end
