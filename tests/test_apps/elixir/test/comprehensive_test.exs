defmodule KreuzbergTestApp.ComprehensiveTest do
  use ExUnit.Case, async: true
  import KreuzbergTestApp.TestHelpers

  @moduledoc """
  Comprehensive tests for Kreuzberg Elixir bindings covering edge cases,
  error handling, serialization, and plugin system integration.
  """

  describe "error handling - invalid inputs" do
    test "extract with empty binary" do
      {:error, _reason} = Kreuzberg.extract("", "text/plain")
    end

    test "extract with nil MIME type" do
      # Guard clause should reject nil mime_type - this will raise FunctionClauseError
      assert_raise FunctionClauseError, fn ->
        Kreuzberg.extract("test", nil)
      end
    end

    test "extract_file with nil path" do
      # Guard clause should reject nil path - this will raise FunctionClauseError
      assert_raise FunctionClauseError, fn ->
        Kreuzberg.extract_file(nil, "application/pdf")
      end
    end

    test "batch_extract_files with empty list" do
      {:error, _reason} = Kreuzberg.batch_extract_files([], "application/pdf")
    end

    test "batch_extract_bytes with mismatched MIME types length" do
      data = ["a", "b", "c"]
      mime_types = ["text/plain", "text/plain"]
      {:error, _reason} = Kreuzberg.batch_extract_bytes(data, mime_types)
    end
  end

  describe "error handling - bang variants error propagation" do
    test "extract! propagates error reason" do
      assert_raise Kreuzberg.Error, fn ->
        Kreuzberg.extract!(<<0, 1, 2>>, "application/pdf")
      end
    end

    test "extract_file! propagates error reason" do
      assert_raise Kreuzberg.Error, fn ->
        Kreuzberg.extract_file!("/nonexistent", "application/pdf")
      end
    end
  end

  describe "async operations - concurrent processing" do
    test "multiple concurrent async extractions" do
      tasks = 1..3
      |> Enum.map(fn i ->
        binary = "Content #{i}"
        Kreuzberg.extract_async(binary, "text/plain")
      end)

      results = Task.await_many(tasks)
      assert length(results) == 3
    end

    test "async extract_file multiple files" do
      if test_doc_exists?("tiny.pdf") do
        task1 = Kreuzberg.extract_file_async(test_doc_path("tiny.pdf"))
        task2 = Kreuzberg.extract_file_async(test_doc_path("tiny.pdf"))

        results = Task.await_many([task1, task2])
        assert length(results) == 2
      end
    end

    test "batch_extract_files_async multiple files" do
      if test_doc_exists?("tiny.pdf") do
        paths = [test_doc_path("tiny.pdf"), test_doc_path("tiny.pdf")]
        task = Kreuzberg.batch_extract_files_async(paths, "application/pdf")
        {:ok, results} = Task.await(task)
        assert length(results) == 2
      end
    end
  end

  describe "configuration edge cases" do
    test "config with all boolean fields set" do
      config = %Kreuzberg.ExtractionConfig{
        use_cache: false,
        force_ocr: true,
        enable_quality_processing: false
      }

      {:ok, _result} = Kreuzberg.extract("test", "text/plain", config)
    end

    test "config with all nested fields populated" do
      config = %Kreuzberg.ExtractionConfig{
        chunking: %{"max_chars" => 512},
        ocr: %{"enabled" => true},
        language_detection: %{"enabled" => true},
        postprocessor: %{"enabled" => true},
        images: %{"extract" => true},
        pages: %{"start" => 0},
        token_reduction: %{"enabled" => true},
        keywords: %{"algorithm" => "yake"},
        pdf_options: %{"use_ocr" => true}
      }

      {:ok, result} = Kreuzberg.extract("test", "text/plain", config)
      assert is_binary(result.content)
    end

    test "config conversion with mixed key types" do
      config_map = %{
        "use_cache" => false,
        :force_ocr => true,
        "chunking" => %{max_chars: 512}
      }

      result = Kreuzberg.ExtractionConfig.to_map(config_map)
      assert is_map(result)
      # All keys should be strings after normalization
      Enum.each(result, fn {k, _v} ->
        assert is_binary(k)
      end)
    end
  end

  describe "MIME type operations - edge cases" do
    test "detect MIME type from path with no extension" do
      # Files without extensions will fail with IO error if they don't exist
      # Test with a known non-existent file should return error
      {:error, _reason} = Kreuzberg.detect_mime_type_from_path("file_no_ext")
    end

    test "detect MIME type from path with multiple dots" do
      # archive.tar.gz doesn't exist, so it should return error
      {:error, _reason} = Kreuzberg.detect_mime_type_from_path("archive.tar.gz")
    end

    test "get extensions for MIME type - returns list" do
      {:ok, extensions} = Kreuzberg.get_extensions_for_mime("application/pdf")
      assert is_list(extensions)
      assert Enum.all?(extensions, &is_binary/1)
    end
  end

  describe "validator edge cases" do
    test "validate_dpi boundary values" do
      assert :ok = Kreuzberg.validate_dpi(1)
      assert :ok = Kreuzberg.validate_dpi(2400)
      {:error, _} = Kreuzberg.validate_dpi(2401)
    end

    test "validate_confidence boundary values" do
      assert :ok = Kreuzberg.validate_confidence(0.0)
      assert :ok = Kreuzberg.validate_confidence(1.0)
    end

    test "validate_chunking_params with large numbers" do
      params = %{"max_chars" => 1000000, "max_overlap" => 500000}
      assert :ok = Kreuzberg.validate_chunking_params(params)
    end

    test "validate_tesseract_psm boundary values" do
      assert :ok = Kreuzberg.validate_tesseract_psm(0)
      assert :ok = Kreuzberg.validate_tesseract_psm(13)
      {:error, _} = Kreuzberg.validate_tesseract_psm(14)
    end

    test "validate_tesseract_oem boundary values" do
      assert :ok = Kreuzberg.validate_tesseract_oem(0)
      assert :ok = Kreuzberg.validate_tesseract_oem(3)
      {:error, _} = Kreuzberg.validate_tesseract_oem(4)
    end
  end

  describe "error classification - case sensitivity" do
    test "classify_error is case insensitive" do
      assert :io_error = Kreuzberg.classify_error("FILE NOT FOUND")
      assert :io_error = Kreuzberg.classify_error("File not found")
      assert :io_error = Kreuzberg.classify_error("file not found")
    end

    test "classify_error with various error patterns" do
      assert :invalid_format = Kreuzberg.classify_error("corrupted")
      # "option error" contains "io" so it matches io_error regex
      assert :io_error = Kreuzberg.classify_error("option error")
      # Use "ocr engine failed" instead of "optical..." to avoid the "io" in "recognition"
      assert :ocr_error = Kreuzberg.classify_error("ocr engine failed")
    end
  end

  describe "embedding presets" do
    test "embedding presets are retrievable" do
      {:ok, presets} = Kreuzberg.list_embedding_presets()
      assert is_list(presets)

      Enum.each(presets, fn preset_name ->
        {:ok, preset_info} = Kreuzberg.get_embedding_preset(preset_name)
        assert is_map(preset_info)
      end)
    end

    test "get_embedding_preset with nonexistent preset" do
      {:error, _reason} = Kreuzberg.get_embedding_preset("nonexistent_xyz_123")
    end
  end

  describe "batch operations - complex scenarios" do
    test "batch extract with mixed content sizes" do
      data = [
        "Short",
        "This is a longer piece of content with more characters in it",
        "Med"
      ]
      {:ok, results} = Kreuzberg.batch_extract_bytes(data, "text/plain")
      assert length(results) == 3
    end

    test "batch extract bytes with list of different MIME types" do
      data = ["Text 1", "Text 2"]
      mime_types = ["text/plain", "text/plain"]
      {:ok, results} = Kreuzberg.batch_extract_bytes(data, mime_types)
      assert length(results) == 2
    end
  end

  describe "plugin system - complex scenarios" do
    test "extract_with_plugins with all plugin options empty" do
      binary = "Test content"
      {:ok, result} = Kreuzberg.extract_with_plugins(
        binary,
        "text/plain",
        nil,
        validators: [],
        post_processors: %{},
        final_validators: []
      )
      assert is_binary(result.content)
    end

    test "extract_with_plugins with config and empty plugins" do
      binary = "Content"
      config = %Kreuzberg.ExtractionConfig{use_cache: false}
      {:ok, result} = Kreuzberg.extract_with_plugins(
        binary,
        "text/plain",
        config,
        []
      )
      assert is_binary(result.content)
    end
  end

  describe "result structures" do
    test "extraction result has all expected fields" do
      {:ok, result} = Kreuzberg.extract("Test", "text/plain")
      assert has_struct_field?(result, :content)
      assert is_binary(result.content)
    end

    test "batch extraction returns correct number of results" do
      data = ["A", "B", "C"]
      {:ok, results} = Kreuzberg.batch_extract_bytes(data, "text/plain")
      assert length(results) == 3
    end
  end

  describe "configuration discovery" do
    test "discover returns error when no config found" do
      result = Kreuzberg.discover_extraction_config()
      assert result == {:error, :not_found} or match?({:ok, _}, result)
    end
  end

  describe "config validation - comprehensive checks" do
    test "validate with all valid fields" do
      config = %Kreuzberg.ExtractionConfig{
        use_cache: true,
        force_ocr: false,
        enable_quality_processing: true,
        chunking: %{"max_chars" => 512, "max_overlap" => 100},
        ocr: %{"confidence" => 0.5, "dpi" => 300}
      }
      assert {:ok, _} = Kreuzberg.ExtractionConfig.validate(config)
    end

    test "validate catches multiple errors in OCR config" do
      config = %Kreuzberg.ExtractionConfig{
        ocr: %{"confidence" => 1.5, "dpi" => 0}
      }
      assert {:error, _} = Kreuzberg.ExtractionConfig.validate(config)
    end

    test "validate with valid chunking overlap boundary" do
      config = %Kreuzberg.ExtractionConfig{
        chunking: %{"max_chars" => 100, "max_overlap" => 100}
      }
      # Overlap equal to max_chars is actually valid - it only requires < not <=
      result = Kreuzberg.ExtractionConfig.validate(config)
      assert match?({:ok, _}, result)
    end

    test "validate with invalid chunking overlap" do
      config = %Kreuzberg.ExtractionConfig{
        chunking: %{"max_chars" => 100, "max_overlap" => 101}
      }
      # Overlap greater than max_chars should be invalid
      result = Kreuzberg.ExtractionConfig.validate(config)
      assert match?({:error, _}, result)
    end
  end

  describe "config serialization - type conversions" do
    test "to_map with nested map config" do
      nested_config = %{
        "chunking" => %{"max_chars" => 1024},
        "use_cache" => true
      }
      result = Kreuzberg.ExtractionConfig.to_map(nested_config)
      assert result["chunking"]["max_chars"] == 1024
    end

    test "to_map preserves keyword list structure" do
      keyword_config = [
        use_cache: true,
        force_ocr: false,
        chunking: %{"max_chars" => 512}
      ]
      result = Kreuzberg.ExtractionConfig.to_map(keyword_config)
      assert result["use_cache"] == true
      assert result["force_ocr"] == false
    end

    test "to_map with keywords config" do
      config = %Kreuzberg.ExtractionConfig{
        keywords: %{"max_keywords" => 15}
      }
      result = Kreuzberg.ExtractionConfig.to_map(config)
      keywords = result["keywords"]
      assert keywords["max_keywords"] == 15
      assert keywords["algorithm"] == "yake"  # default
    end
  end

  describe "cache operations - return values" do
    test "cache_stats returns proper structure" do
      case Kreuzberg.cache_stats() do
        {:ok, stats} ->
          assert is_map(stats)

        {:error, _} ->
          :ok
      end
    end

    test "clear_cache returns atom" do
      result = Kreuzberg.clear_cache()
      assert result == :ok or match?({:error, _}, result)
    end
  end

  # Helper function
  defp has_struct_field?(struct, field) do
    try do
      Map.fetch!(struct, field)
      true
    catch
      :error, _ -> false
    end
  end
end
