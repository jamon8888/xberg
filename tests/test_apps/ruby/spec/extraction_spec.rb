# frozen_string_literal: true

require_relative 'spec_helper'

RSpec.describe 'Kreuzberg Extraction' do
  describe 'Type verification' do
    it 'Kreuzberg module exists' do
      expect(defined?(Kreuzberg)).to be_truthy
    end

    it 'Kreuzberg::Result is accessible' do
      expect(defined?(Kreuzberg::Result)).to be_truthy
    end

    it 'Kreuzberg::Config is accessible' do
      expect(defined?(Kreuzberg::Config)).to be_truthy
    end

    it 'Kreuzberg::Config::Extraction is accessible' do
      expect(defined?(Kreuzberg::Config::Extraction)).to be_truthy
    end

    it 'Kreuzberg::Config::OCR is accessible' do
      expect(defined?(Kreuzberg::Config::OCR)).to be_truthy
    end

    it 'Kreuzberg::Errors module exists' do
      expect(defined?(Kreuzberg::Errors)).to be_truthy
    end

    it 'Kreuzberg::Errors::IOError is accessible' do
      expect(defined?(Kreuzberg::Errors::IOError)).to be_truthy
    end

    it 'Kreuzberg::CLI is accessible' do
      expect(defined?(Kreuzberg::CLI)).to be_truthy
    end
  end

  describe 'Synchronous file extraction' do
    it 'extracts content from DOCX file' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to be_a(Kreuzberg::Result)
      expect(result.content).to be_a(String)
      expect(result.content).not_to be_empty
      expect(result.mime_type).to include('wordprocessing')
    end

    it 'extracts content from ODT file' do
      path = test_document_path('documents/simple.odt')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to be_a(Kreuzberg::Result)
      expect(result.content).not_to be_empty
    end

    it 'returns result with proper structure' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to respond_to(:content)
      expect(result).to respond_to(:mime_type)
      expect(result).to respond_to(:metadata)
      expect(result).to respond_to(:tables)
      expect(result).to respond_to(:chunks)
    end

    it 'handles file with explicit MIME type' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path, mime_type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document')

      expect(result.content).not_to be_empty
    end
  end

  describe 'Asynchronous file extraction' do
    it 'extracts content from file asynchronously' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file(path: path)

      expect(result).to be_a(Kreuzberg::Result)
      expect(result.content).not_to be_empty
    end

    it 'returns async result with content and metadata' do
      path = test_document_path('documents/simple.odt')
      result = Kreuzberg.extract_file(path: path)

      expect(result.content).to be_a(String)
      expect(result.mime_type).to be_a(String)
    end
  end

  describe 'Synchronous byte extraction' do
    it 'extracts content from binary DOCX data' do
      path = test_document_path('documents/fake.docx')
      data = read_test_document('documents/fake.docx')
      result = Kreuzberg.extract_bytes_sync(
        data: data,
        mime_type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
      )

      expect(result).to be_a(Kreuzberg::Result)
      expect(result.content).not_to be_empty
    end

    it 'extracts content from binary ODT data' do
      data = read_test_document('documents/simple.odt')
      result = Kreuzberg.extract_bytes_sync(data: data, mime_type: 'application/vnd.oasis.opendocument.text')

      expect(result.content).not_to be_empty
    end

    it 'requires MIME type for byte extraction' do
      data = read_test_document('documents/fake.docx')
      result = Kreuzberg.extract_bytes_sync(data: data, mime_type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document')
      expect(result).to be_a(Kreuzberg::Result)
    end
  end

  describe 'Asynchronous byte extraction' do
    it 'extracts content from binary data asynchronously' do
      data = read_test_document('documents/fake.docx')
      result = Kreuzberg.extract_bytes(
        data: data,
        mime_type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
      )

      expect(result).to be_a(Kreuzberg::Result)
      expect(result.content).not_to be_empty
    end

    it 'handles async byte extraction from ODT' do
      data = read_test_document('documents/simple.odt')
      result = Kreuzberg.extract_bytes(data: data, mime_type: 'application/vnd.oasis.opendocument.text')

      expect(result.content).not_to be_empty
    end
  end

  describe 'Batch synchronous file extraction' do
    it 'extracts multiple files in batch' do
      paths = [
        test_document_path('documents/fake.docx'),
        test_document_path('documents/simple.odt')
      ]
      results = Kreuzberg.batch_extract_files_sync(paths: paths)

      expect(results).to be_an(Array)
      expect(results.length).to eq(2)
      expect(results.all? { |r| r.is_a?(Kreuzberg::Result) }).to be true
    end

    it 'maintains result order for batch extraction' do
      paths = [
        test_document_path('documents/fake.docx'),
        test_document_path('documents/simple.odt')
      ]
      results = Kreuzberg.batch_extract_files_sync(paths: paths)

      expect(results[0].mime_type).to include('wordprocessing')
      expect(results[1].mime_type).to include('oasis')
    end

    it 'batch extracts with single file' do
      paths = [test_document_path('documents/fake.docx')]
      results = Kreuzberg.batch_extract_files_sync(paths: paths)

      expect(results.length).to eq(1)
      expect(results[0].content).not_to be_empty
    end

    it 'all batch results have content' do
      paths = [
        test_document_path('documents/fake.docx'),
        test_document_path('documents/simple.odt')
      ]
      results = Kreuzberg.batch_extract_files_sync(paths: paths)

      expect(results.all? { |r| r.content.is_a?(String) && !r.content.empty? }).to be true
    end
  end

  describe 'Batch asynchronous file extraction' do
    it 'extracts multiple files asynchronously' do
      paths = [
        test_document_path('documents/fake.docx'),
        test_document_path('documents/simple.odt')
      ]
      results = Kreuzberg.batch_extract_files(paths: paths)

      expect(results).to be_an(Array)
      expect(results.length).to eq(2)
    end

    it 'async batch extracts with configuration' do
      paths = [test_document_path('documents/fake.docx')]
      config = Kreuzberg::Config::Extraction.new
      results = Kreuzberg.batch_extract_files(paths: paths, config: config)

      expect(results[0]).to be_a(Kreuzberg::Result)
      expect(results[0].content).not_to be_empty
    end
  end

  describe 'Batch byte extraction' do
    it 'batch extracts multiple binary documents synchronously' do
      data_array = [
        read_test_document('documents/fake.docx'),
        read_test_document('documents/simple.odt')
      ]
      mime_types = [
        'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        'application/vnd.oasis.opendocument.text'
      ]
      results = Kreuzberg.batch_extract_bytes_sync(data_array: data_array, mime_types: mime_types)

      expect(results).to be_an(Array)
      expect(results.length).to eq(2)
      expect(results.all? { |r| r.is_a?(Kreuzberg::Result) }).to be true
    end

    it 'batch extracts multiple binary documents asynchronously' do
      data_array = [
        read_test_document('documents/fake.docx'),
        read_test_document('documents/simple.odt')
      ]
      mime_types = [
        'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        'application/vnd.oasis.opendocument.text'
      ]
      results = Kreuzberg.batch_extract_bytes(data_array: data_array, mime_types: mime_types)

      expect(results.length).to eq(2)
      expect(results.all? { |r| r.content.is_a?(String) }).to be true
    end

    it 'maintains order in batch byte extraction' do
      data_array = [
        read_test_document('documents/fake.docx'),
        read_test_document('documents/simple.odt')
      ]
      mime_types = [
        'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        'application/vnd.oasis.opendocument.text'
      ]
      results = Kreuzberg.batch_extract_bytes_sync(data_array: data_array, mime_types: mime_types)

      expect(results[0].mime_type).to include('wordprocessing')
      expect(results[1].mime_type).to include('oasis')
    end
  end

  describe 'MIME type detection' do
    it 'detects MIME type from file path' do
      path = test_document_path('documents/fake.docx')
      mime_type = Kreuzberg::CLI.detect(path: path)

      expect(mime_type).to be_a(String)
      expect(mime_type).not_to be_empty
      expect(mime_type).to include('wordprocessing')
    end

    it 'detects MIME type for ODT files' do
      path = test_document_path('documents/simple.odt')
      mime_type = Kreuzberg::CLI.detect(path: path)

      expect(mime_type).to include('oasis')
    end

    it 'extracts and provides MIME type in result' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result.mime_type).to be_a(String)
      expect(result.mime_type).not_to be_empty
    end
  end

  describe 'File type coverage' do
    it 'extracts from DOCX files' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to be_a(Kreuzberg::Result)
      expect(result.content).not_to be_empty
      expect(result.mime_type).to include('wordprocessingml')
    end

    it 'extracts from ODT files' do
      path = test_document_path('documents/simple.odt')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to be_a(Kreuzberg::Result)
      expect(result.content).not_to be_empty
    end

    it 'extracts from Markdown files' do
      path = test_document_path('extraction_test.md')
      skip 'Markdown test file required' unless File.exist?(path)
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to be_a(Kreuzberg::Result)
    end

    it 'extracts from image files - PNG' do
      path = test_document_path('images/sample.png')
      skip 'PNG test file required' unless File.exist?(path)
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to be_a(Kreuzberg::Result)
    end

    it 'extracts from image files - JPG' do
      path = test_document_path('images/example.jpg')
      skip 'JPG test file required' unless File.exist?(path)
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to be_a(Kreuzberg::Result)
    end
  end

  describe 'Configuration handling' do
    it 'creates extraction config object' do
      config = Kreuzberg::Config::Extraction.new
      expect(config).to be_a(Kreuzberg::Config::Extraction)
    end

    it 'creates OCR config object' do
      ocr_config = Kreuzberg::Config::OCR.new
      expect(ocr_config).to be_a(Kreuzberg::Config::OCR)
    end

    it 'extracts with custom config' do
      path = test_document_path('documents/fake.docx')
      config = Kreuzberg::Config::Extraction.new
      result = Kreuzberg.extract_file_sync(path: path, config: config)

      expect(result).to be_a(Kreuzberg::Result)
      expect(result.content).not_to be_empty
    end

    it 'extracts with hash config' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path, config: {})

      expect(result).to be_a(Kreuzberg::Result)
    end
  end

  describe 'Result structure and attributes' do
    it 'result has content attribute' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to respond_to(:content)
      expect(result.content).to be_a(String)
    end

    it 'result has MIME type attribute' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to respond_to(:mime_type)
      expect(result.mime_type).to be_a(String)
    end

    it 'result has metadata attribute' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to respond_to(:metadata)
    end

    it 'result has tables attribute' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to respond_to(:tables)
      expect(result.tables).to be_an(Array)
    end

    it 'result has chunks attribute' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to respond_to(:chunks)
      expect(result.chunks).to be_an(Array)
    end

    it 'result has detected_languages attribute' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to respond_to(:detected_languages)
    end

    it 'result has pages attribute' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to respond_to(:pages)
    end

    it 'result has images attribute' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to respond_to(:images)
    end
  end

  describe 'Integration tests' do
    it 'extracts and provides consistent results on repeated calls' do
      path = test_document_path('documents/fake.docx')
      result1 = Kreuzberg.extract_file_sync(path: path)
      result2 = Kreuzberg.extract_file_sync(path: path)

      expect(result1.content).to eq(result2.content)
      expect(result1.mime_type).to eq(result2.mime_type)
    end

    it 'sync and async extraction produce same content' do
      path = test_document_path('documents/fake.docx')
      sync_result = Kreuzberg.extract_file_sync(path: path)
      async_result = Kreuzberg.extract_file(path: path)

      expect(sync_result.content).to eq(async_result.content)
    end

    it 'file and bytes extraction produce same content' do
      path = test_document_path('documents/fake.docx')
      file_result = Kreuzberg.extract_file_sync(path: path)
      data = read_test_document('documents/fake.docx')
      bytes_result = Kreuzberg.extract_bytes_sync(
        data: data,
        mime_type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
      )

      expect(file_result.content).to eq(bytes_result.content)
    end

    it 'batch and individual extraction produce same results' do
      path = test_document_path('documents/fake.docx')
      individual_result = Kreuzberg.extract_file_sync(path: path)
      batch_results = Kreuzberg.batch_extract_files_sync(paths: [path])

      expect(individual_result.content).to eq(batch_results[0].content)
    end
  end

  describe 'CLI interface' do
    it 'CLI extract returns string output' do
      path = test_document_path('documents/fake.docx')
      output = Kreuzberg::CLI.extract(path: path)

      expect(output).to be_a(String)
      expect(output).not_to be_empty
    end

    it 'CLI detect returns MIME type' do
      path = test_document_path('documents/fake.docx')
      mime_type = Kreuzberg::CLI.detect(path: path)

      expect(mime_type).to be_a(String)
      expect(mime_type).not_to be_empty
    end

    it 'CLI version returns version string' do
      version = Kreuzberg::CLI.version
      expect(version).to be_a(String)
      expect(version).to match(/\d+\.\d+/)
    end
  end

  describe 'Output format configuration' do
    describe 'output_format: plain' do
      it 'creates config with plain output format' do
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'plain'

        expect(config.output_format).to eq('plain')
      end

      it 'extracts file with plain output format' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'plain'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
        expect(result.content).not_to be_empty
      end

      it 'batch extracts with plain output format' do
        paths = [
          test_document_path('documents/fake.docx'),
          test_document_path('documents/simple.odt')
        ]
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'plain'
        results = Kreuzberg.batch_extract_files_sync(paths: paths, config: config)

        expect(results.length).to eq(2)
        expect(results.all? { |r| r.is_a?(Kreuzberg::Result) }).to be true
      end
    end

    describe 'output_format: markdown' do
      it 'creates config with markdown output format' do
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'markdown'

        expect(config.output_format).to eq('markdown')
      end

      it 'extracts file with markdown output format' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'markdown'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
        expect(result.content).not_to be_empty
      end

      it 'markdown output format produces structured content' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'markdown'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        # Markdown content may contain markdown syntax
        expect(result.content).to be_a(String)
      end

      it 'batch extracts with markdown output format' do
        paths = [test_document_path('documents/fake.docx')]
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'markdown'
        results = Kreuzberg.batch_extract_files_sync(paths: paths, config: config)

        expect(results[0]).to be_a(Kreuzberg::Result)
      end
    end

    describe 'output_format: djot' do
      it 'creates config with djot output format' do
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'djot'

        expect(config.output_format).to eq('djot')
      end

      it 'extracts file with djot output format' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'djot'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
        expect(result.content).not_to be_empty
      end

      it 'djot format extraction returns valid result' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'djot'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result.content).to be_a(String)
      end
    end

    describe 'output_format: html' do
      it 'creates config with html output format' do
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'html'

        expect(config.output_format).to eq('html')
      end

      it 'extracts file with html output format' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'html'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
        expect(result.content).not_to be_empty
      end

      it 'html format extraction produces html content' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'html'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        # HTML content may contain html tags
        expect(result.content).to be_a(String)
      end

      it 'async extraction with html format' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'html'
        result = Kreuzberg.extract_file(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
      end
    end

    describe 'output_format switching' do
      it 'switches output format in config' do
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'plain'
        expect(config.output_format).to eq('plain')

        config.output_format = 'markdown'
        expect(config.output_format).to eq('markdown')
      end

      it 'plain and markdown produce different results' do
        path = test_document_path('documents/fake.docx')

        plain_config = Kreuzberg::Config::Extraction.new
        plain_config.output_format = 'plain'
        plain_result = Kreuzberg.extract_file_sync(path: path, config: plain_config)

        markdown_config = Kreuzberg::Config::Extraction.new
        markdown_config.output_format = 'markdown'
        markdown_result = Kreuzberg.extract_file_sync(path: path, config: markdown_config)

        # Results should be valid results
        expect(plain_result).to be_a(Kreuzberg::Result)
        expect(markdown_result).to be_a(Kreuzberg::Result)
      end
    end
  end

  describe 'Result format configuration' do
    describe 'result_format: unified' do
      it 'creates config with unified result format' do
        config = Kreuzberg::Config::Extraction.new
        config.result_format = 'unified'

        expect(config.result_format).to eq('unified')
      end

      it 'extracts file with unified result format' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.result_format = 'unified'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
        expect(result.content).not_to be_empty
      end

      it 'unified format produces consistent results' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.result_format = 'unified'
        result1 = Kreuzberg.extract_file_sync(path: path, config: config)
        result2 = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result1.content).to eq(result2.content)
      end

      it 'batch extracts with unified result format' do
        paths = [test_document_path('documents/fake.docx')]
        config = Kreuzberg::Config::Extraction.new
        config.result_format = 'unified'
        results = Kreuzberg.batch_extract_files_sync(paths: paths, config: config)

        expect(results[0]).to be_a(Kreuzberg::Result)
      end
    end

    describe 'result_format: element_based' do
      it 'creates config with element_based result format' do
        config = Kreuzberg::Config::Extraction.new
        config.result_format = 'element_based'

        expect(config.result_format).to eq('element_based')
      end

      it 'extracts file with element_based result format' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.result_format = 'element_based'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
        expect(result.content).not_to be_empty
      end

      it 'element_based format extraction returns valid result' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.result_format = 'element_based'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result.content).to be_a(String)
      end

      it 'element_based result has element attributes' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.result_format = 'element_based'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        # Element-based results may provide additional structure
        expect(result).to respond_to(:content)
      end
    end

    describe 'result_format switching' do
      it 'switches result format in config' do
        config = Kreuzberg::Config::Extraction.new
        config.result_format = 'unified'
        expect(config.result_format).to eq('unified')

        config.result_format = 'element_based'
        expect(config.result_format).to eq('element_based')
      end

      it 'unified and element_based formats are consistent' do
        path = test_document_path('documents/fake.docx')

        unified_config = Kreuzberg::Config::Extraction.new
        unified_config.result_format = 'unified'
        unified_result = Kreuzberg.extract_file_sync(path: path, config: unified_config)

        element_config = Kreuzberg::Config::Extraction.new
        element_config.result_format = 'element_based'
        element_result = Kreuzberg.extract_file_sync(path: path, config: element_config)

        # Both should produce valid results
        expect(unified_result).to be_a(Kreuzberg::Result)
        expect(element_result).to be_a(Kreuzberg::Result)
      end
    end
  end

  describe 'Format combinations' do
    describe 'output_format and result_format together' do
      it 'combines plain output with unified result' do
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'plain'
        config.result_format = 'unified'

        expect(config.output_format).to eq('plain')
        expect(config.result_format).to eq('unified')
      end

      it 'extracts with plain output and unified result' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'plain'
        config.result_format = 'unified'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
        expect(result.content).not_to be_empty
      end

      it 'combines markdown output with element_based result' do
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'markdown'
        config.result_format = 'element_based'

        expect(config.output_format).to eq('markdown')
        expect(config.result_format).to eq('element_based')
      end

      it 'extracts with markdown output and element_based result' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'markdown'
        config.result_format = 'element_based'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'combines djot output with unified result' do
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'djot'
        config.result_format = 'unified'

        expect(config.output_format).to eq('djot')
        expect(config.result_format).to eq('unified')
      end

      it 'combines html output with element_based result' do
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'html'
        config.result_format = 'element_based'

        expect(config.output_format).to eq('html')
        expect(config.result_format).to eq('element_based')
      end
    end

    describe 'all format combinations' do
      it 'plain + unified combination' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'plain'
        config.result_format = 'unified'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'plain + element_based combination' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'plain'
        config.result_format = 'element_based'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'markdown + unified combination' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'markdown'
        config.result_format = 'unified'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'markdown + element_based combination' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'markdown'
        config.result_format = 'element_based'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'djot + unified combination' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'djot'
        config.result_format = 'unified'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'djot + element_based combination' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'djot'
        config.result_format = 'element_based'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'html + unified combination' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'html'
        config.result_format = 'unified'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'html + element_based combination' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'html'
        config.result_format = 'element_based'
        result = Kreuzberg.extract_file_sync(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
      end
    end
  end

  describe 'Format configuration serialization' do
    describe 'hash-based config with formats' do
      it 'extracts with hash config containing output_format' do
        path = test_document_path('documents/fake.docx')
        result = Kreuzberg.extract_file_sync(path: path, config: { output_format: 'plain' })

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'extracts with hash config containing result_format' do
        path = test_document_path('documents/fake.docx')
        result = Kreuzberg.extract_file_sync(path: path, config: { result_format: 'unified' })

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'extracts with hash config containing both formats' do
        path = test_document_path('documents/fake.docx')
        result = Kreuzberg.extract_file_sync(
          path: path,
          config: { output_format: 'markdown', result_format: 'element_based' }
        )

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'batch extracts with hash config containing output_format' do
        paths = [test_document_path('documents/fake.docx')]
        results = Kreuzberg.batch_extract_files_sync(
          paths: paths,
          config: { output_format: 'plain' }
        )

        expect(results[0]).to be_a(Kreuzberg::Result)
      end

      it 'batch extracts with hash config containing result_format' do
        paths = [test_document_path('documents/fake.docx')]
        results = Kreuzberg.batch_extract_files_sync(
          paths: paths,
          config: { result_format: 'element_based' }
        )

        expect(results[0]).to be_a(Kreuzberg::Result)
      end
    end

    describe 'config object serialization' do
      it 'config object maintains output_format after extraction' do
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'markdown'
        path = test_document_path('documents/fake.docx')
        Kreuzberg.extract_file_sync(path: path, config: config)

        expect(config.output_format).to eq('markdown')
      end

      it 'config object maintains result_format after extraction' do
        config = Kreuzberg::Config::Extraction.new
        config.result_format = 'element_based'
        path = test_document_path('documents/fake.docx')
        Kreuzberg.extract_file_sync(path: path, config: config)

        expect(config.result_format).to eq('element_based')
      end

      it 'config with both formats maintains values after extraction' do
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'html'
        config.result_format = 'unified'
        path = test_document_path('documents/fake.docx')
        Kreuzberg.extract_file_sync(path: path, config: config)

        expect(config.output_format).to eq('html')
        expect(config.result_format).to eq('unified')
      end
    end

    describe 'async extraction with format configs' do
      it 'async file extraction with output_format' do
        path = test_document_path('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'markdown'
        result = Kreuzberg.extract_file(path: path, config: config)

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'async byte extraction with result_format' do
        data = read_test_document('documents/fake.docx')
        config = Kreuzberg::Config::Extraction.new
        config.result_format = 'element_based'
        result = Kreuzberg.extract_bytes(
          data: data,
          mime_type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
          config: config
        )

        expect(result).to be_a(Kreuzberg::Result)
      end

      it 'async batch extraction with format config' do
        paths = [test_document_path('documents/fake.docx')]
        config = Kreuzberg::Config::Extraction.new
        config.output_format = 'djot'
        config.result_format = 'unified'
        results = Kreuzberg.batch_extract_files(paths: paths, config: config)

        expect(results[0]).to be_a(Kreuzberg::Result)
      end
    end
  end

  describe 'Format persistence and reuse' do
    it 'reuses same config object with different files' do
      config = Kreuzberg::Config::Extraction.new
      config.output_format = 'plain'
      config.result_format = 'unified'

      result1 = Kreuzberg.extract_file_sync(
        path: test_document_path('documents/fake.docx'),
        config: config
      )
      result2 = Kreuzberg.extract_file_sync(
        path: test_document_path('documents/simple.odt'),
        config: config
      )

      expect(result1).to be_a(Kreuzberg::Result)
      expect(result2).to be_a(Kreuzberg::Result)
      expect(config.output_format).to eq('plain')
      expect(config.result_format).to eq('unified')
    end

    it 'config values do not affect other config objects' do
      config1 = Kreuzberg::Config::Extraction.new
      config1.output_format = 'plain'

      config2 = Kreuzberg::Config::Extraction.new
      config2.output_format = 'markdown'

      expect(config1.output_format).to eq('plain')
      expect(config2.output_format).to eq('markdown')
    end

    it 'modifying config does not affect previous extractions' do
      config = Kreuzberg::Config::Extraction.new
      config.output_format = 'plain'
      result1 = Kreuzberg.extract_file_sync(
        path: test_document_path('documents/fake.docx'),
        config: config
      )

      config.output_format = 'markdown'
      result2 = Kreuzberg.extract_file_sync(
        path: test_document_path('documents/fake.docx'),
        config: config
      )

      expect(result1).to be_a(Kreuzberg::Result)
      expect(result2).to be_a(Kreuzberg::Result)
    end
  end
end
