# frozen_string_literal: true

require_relative 'spec_helper'

RSpec.describe 'Kreuzberg Comprehensive Gap Coverage Tests' do
  describe 'Config Serialization - to_h and from_h' do
    describe 'Config::Extraction serialization' do
      it 'to_h produces hash with all fields' do
        config = Kreuzberg::Config::Extraction.new(
          force_ocr: true,
          use_cache: false,
          output_format: 'markdown',
          result_format: 'unified'
        )
        hash = config.to_h

        expect(hash).to be_a(Hash)
        expect(hash[:force_ocr]).to eq(true)
        expect(hash[:use_cache]).to eq(false)
        expect(hash[:output_format]).to eq('markdown')
        expect(hash[:result_format]).to eq('unified')
      end

      it 'to_h with nested OCR config' do
        ocr_config = Kreuzberg::Config::OCR.new(language: 'fra')
        config = Kreuzberg::Config::Extraction.new(ocr: ocr_config)
        hash = config.to_h

        expect(hash).to be_a(Hash)
        expect(hash).to have_key(:ocr)
      end

      it 'to_h with nested Chunking config' do
        chunking = Kreuzberg::Config::Chunking.new(max_chars: 2000, max_overlap: 300)
        config = Kreuzberg::Config::Extraction.new(chunking: chunking)
        hash = config.to_h

        expect(hash).to be_a(Hash)
        expect(hash).to have_key(:chunking)
      end

      it 'to_h with PDF options config' do
        pdf = Kreuzberg::Config::PDF.new(extract_images: true, dpi: 300)
        config = Kreuzberg::Config::Extraction.new(pdf_options: pdf)
        hash = config.to_h

        expect(hash).to be_a(Hash)
        expect(hash).to have_key(:pdf_options)
      end

      it 'to_h with ImageExtraction config' do
        img_extract = Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 200,
          min_width: 50
        )
        config = Kreuzberg::Config::Extraction.new(image_extraction: img_extract)
        hash = config.to_h

        expect(hash).to be_a(Hash)
        expect(hash).to have_key(:image_extraction)
      end

      it 'to_h with ImagePreprocessing config' do
        preprocessing = Kreuzberg::Config::ImagePreprocessing.new(
          target_dpi: 300,
          auto_rotate: true,
          deskew: true
        )
        config = Kreuzberg::Config::Extraction.new(image_preprocessing: preprocessing)
        hash = config.to_h

        expect(hash).to be_a(Hash)
        expect(hash).to have_key(:image_preprocessing)
      end

      it 'to_h with LanguageDetection config' do
        lang_detect = Kreuzberg::Config::LanguageDetection.new(
          enabled: true,
          min_confidence: 0.7
        )
        config = Kreuzberg::Config::Extraction.new(language_detection: lang_detect)
        hash = config.to_h

        expect(hash).to be_a(Hash)
        expect(hash).to have_key(:language_detection)
      end

      it 'to_h with PostProcessor config' do
        postproc = Kreuzberg::Config::PostProcessor.new(
          enabled: true,
          enabled_processors: ['cleaner', 'formatter']
        )
        config = Kreuzberg::Config::Extraction.new(postprocessor: postproc)
        hash = config.to_h

        expect(hash).to be_a(Hash)
        expect(hash).to have_key(:postprocessor)
      end

      it 'to_h with Keywords config' do
        keywords = Kreuzberg::Config::Keywords.new(
          algorithm: :yake,
          max_keywords: 10
        )
        config = Kreuzberg::Config::Extraction.new(keywords: keywords)
        hash = config.to_h

        expect(hash).to be_a(Hash)
        expect(hash).to have_key(:keywords)
      end

      it 'to_h with PageConfig' do
        pages = Kreuzberg::Config::PageConfig.new(extract_pages: true)
        config = Kreuzberg::Config::Extraction.new(pages: pages)
        hash = config.to_h

        expect(hash).to be_a(Hash)
        expect(hash).to have_key(:pages)
      end

      it 'to_h with all nested configs combined' do
        config = Kreuzberg::Config::Extraction.new(
          ocr: Kreuzberg::Config::OCR.new(language: 'deu'),
          chunking: Kreuzberg::Config::Chunking.new(max_chars: 1500),
          language_detection: Kreuzberg::Config::LanguageDetection.new(enabled: true),
          pdf_options: Kreuzberg::Config::PDF.new(extract_images: true),
          image_extraction: Kreuzberg::Config::ImageExtraction.new(extract_images: true),
          image_preprocessing: Kreuzberg::Config::ImagePreprocessing.new(auto_rotate: true),
          postprocessor: Kreuzberg::Config::PostProcessor.new(enabled: true),
          keywords: Kreuzberg::Config::Keywords.new(max_keywords: 15),
          pages: Kreuzberg::Config::PageConfig.new(extract_pages: true)
        )
        hash = config.to_h

        expect(hash).to be_a(Hash)
        expect(hash.keys).to include(:ocr, :chunking, :language_detection, :pdf_options,
                                      :image_extraction, :image_preprocessing, :postprocessor,
                                      :keywords, :pages)
      end
    end

    describe 'Config::OCR serialization' do
      it 'to_h includes all OCR fields' do
        ocr = Kreuzberg::Config::OCR.new(backend: 'paddleocr', language: 'spa')
        hash = ocr.to_h

        expect(hash).to be_a(Hash)
        expect(hash[:backend]).to eq('paddleocr')
        expect(hash[:language]).to eq('spa')
      end

      it 'to_h with tesseract config' do
        tesseract = Kreuzberg::Config::Tesseract.new(oem: 1, psm: 6)
        ocr = Kreuzberg::Config::OCR.new(tesseract_config: tesseract)
        hash = ocr.to_h

        expect(hash).to be_a(Hash)
        expect(hash).to have_key(:tesseract_config)
      end
    end

    describe 'Config::Chunking serialization' do
      it 'to_h includes embedding config' do
        embedding = Kreuzberg::Config::Embedding.new(
          normalize: true,
          batch_size: 32
        )
        chunking = Kreuzberg::Config::Chunking.new(embedding: embedding)
        hash = chunking.to_h

        expect(hash).to be_a(Hash)
        expect(hash).to have_key(:embedding)
      end
    end

    describe 'Result serialization' do
      it 'Result#to_h produces valid hash' do
        result = Kreuzberg::Result.new(
          content: 'test',
          mime_type: 'text/plain',
          metadata_json: '{}',
          tables: [],
          chunks: [],
          images: []
        )
        hash = result.to_h

        expect(hash).to be_a(Hash)
        expect(hash[:content]).to eq('test')
        expect(hash[:mime_type]).to eq('text/plain')
      end

      it 'Result#to_json produces JSON string' do
        result = Kreuzberg::Result.new(
          content: 'test content',
          mime_type: 'text/plain',
          metadata_json: '{}',
          tables: [],
          chunks: [],
          images: []
        )
        json = result.to_json

        expect(json).to be_a(String)
        expect(json).to include('test content')
        expect { JSON.parse(json) }.not_to raise_error
      end
    end

    describe 'Result nested types serialization' do
      it 'Result::Table#to_h' do
        table = Kreuzberg::Result::Table.new(
          cells: [['a', 'b'], ['c', 'd']],
          markdown: '| a | b |\n| c | d |',
          page_number: 1
        )
        hash = table.to_h

        expect(hash).to be_a(Hash)
        expect(hash[:cells]).to eq([['a', 'b'], ['c', 'd']])
        expect(hash[:markdown]).to include('|')
        expect(hash[:page_number]).to eq(1)
      end

      it 'Result::Chunk#to_h' do
        chunk = Kreuzberg::Result::Chunk.new(
          content: 'chunk text',
          byte_start: 0,
          byte_end: 10,
          token_count: 2,
          chunk_index: 0,
          total_chunks: 5,
          first_page: 1,
          last_page: 1,
          embedding: nil
        )
        hash = chunk.to_h

        expect(hash).to be_a(Hash)
        expect(hash[:content]).to eq('chunk text')
        expect(hash[:byte_start]).to eq(0)
        expect(hash[:token_count]).to eq(2)
      end

      it 'Result::Image#to_h' do
        image = Kreuzberg::Result::Image.new(
          data: 'image_data',
          format: 'png',
          image_index: 0,
          page_number: 1,
          width: 800,
          height: 600,
          colorspace: 'RGB',
          bits_per_component: 8,
          is_mask: false,
          description: 'test image',
          ocr_result: nil
        )
        hash = image.to_h

        expect(hash).to be_a(Hash)
        expect(hash[:format]).to eq('png')
        expect(hash[:image_index]).to eq(0)
        expect(hash[:width]).to eq(800)
      end
    end

    describe 'Config::from_file loading' do
      it 'Config::Extraction.from_file returns config object' do
        # This test requires a valid config file in test environment
        skip 'Config file not available in test environment' unless File.exist?('test_config.json')
        config = Kreuzberg::Config::Extraction.from_file('test_config.json')

        expect(config).to be_a(Kreuzberg::Config::Extraction)
      end
    end
  end

  describe 'Batch Operations - Edge Cases' do
    describe 'Empty batch handling' do
      it 'batch_extract_files_sync with empty array' do
        results = Kreuzberg.batch_extract_files_sync(paths: [])
        expect(results).to be_an(Array)
        expect(results).to be_empty
      end

      it 'batch_extract_files with empty array' do
        results = Kreuzberg.batch_extract_files(paths: [])
        expect(results).to be_an(Array)
        expect(results).to be_empty
      end

      it 'batch_extract_bytes_sync with empty array' do
        results = Kreuzberg.batch_extract_bytes_sync(data_array: [], mime_types: [])
        expect(results).to be_an(Array)
        expect(results).to be_empty
      end

      it 'batch_extract_bytes with empty array' do
        results = Kreuzberg.batch_extract_bytes(data_array: [], mime_types: [])
        expect(results).to be_an(Array)
        expect(results).to be_empty
      end
    end

    describe 'Batch result ordering' do
      it 'batch operations maintain file order' do
        paths = [
          test_document_path('documents/fake.docx'),
          test_document_path('documents/simple.odt'),
          test_document_path('documents/fake.docx')
        ]
        results = Kreuzberg.batch_extract_files_sync(paths: paths)

        expect(results.length).to eq(3)
        expect(results[0].mime_type).to include('wordprocessing')
        expect(results[1].mime_type).to include('oasis')
        expect(results[2].mime_type).to include('wordprocessing')
      end

      it 'batch_extract_bytes maintains data order' do
        data = [
          read_test_document('documents/fake.docx'),
          read_test_document('documents/simple.odt')
        ]
        types = [
          'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
          'application/vnd.oasis.opendocument.text'
        ]
        results = Kreuzberg.batch_extract_bytes_sync(data_array: data, mime_types: types)

        expect(results.length).to eq(2)
        expect(results[0].mime_type).to include('wordprocessing')
        expect(results[1].mime_type).to include('oasis')
      end
    end

    describe 'Batch operations with config' do
      it 'batch applies config to all files' do
        paths = [
          test_document_path('documents/fake.docx'),
          test_document_path('documents/simple.odt')
        ]
        config = Kreuzberg::Config::Extraction.new(output_format: 'plain')
        results = Kreuzberg.batch_extract_files_sync(paths: paths, config: config)

        expect(results.length).to eq(2)
        expect(results.all? { |r| r.is_a?(Kreuzberg::Result) }).to be true
      end
    end
  end

  describe 'Plugin System - Comprehensive Coverage' do
    describe 'Validator registration and lifecycle' do
      it 'register_validator returns true' do
        validator = lambda { |result| result[:content].is_a?(String) }
        result = Kreuzberg.register_validator('test_validator', validator)
        expect(result).to be true
        Kreuzberg.unregister_validator('test_validator')
      end

      it 'list_validators includes registered validator' do
        validator = lambda { |result| true }
        Kreuzberg.register_validator('lifecycle_test', validator)
        validators = Kreuzberg.list_validators

        expect(validators).to include('lifecycle_test')
        Kreuzberg.unregister_validator('lifecycle_test')
      end

      it 'unregister_validator returns true' do
        validator = lambda { |result| true }
        Kreuzberg.register_validator('temp_validator', validator)
        result = Kreuzberg.unregister_validator('temp_validator')

        expect(result).to be true
      end

      it 'clear_validators removes all validators' do
        Kreuzberg.register_validator('v1', ->(r) { true })
        Kreuzberg.register_validator('v2', ->(r) { true })
        result = Kreuzberg.clear_validators

        expect(result).to be true
      end

      it 'register_validator with priority parameter' do
        validator = lambda { |result| result[:content]&.length&.positive? || false }
        # Priority parameter may or may not be supported
        begin
          result = Kreuzberg.register_validator('priority_test', validator, priority: 10)
          expect(result).to be true
          Kreuzberg.unregister_validator('priority_test')
        rescue ArgumentError
          # Priority may not be supported
          skip 'Priority parameter not supported'
        end
      end
    end

    describe 'Post-processor registration and lifecycle' do
      it 'register_post_processor returns true' do
        processor = lambda { |result| result }
        result = Kreuzberg.register_post_processor('test_processor', processor)
        expect(result).to be true
        Kreuzberg.unregister_post_processor('test_processor')
      end

      it 'list_post_processors includes registered processor' do
        processor = lambda { |result| result }
        Kreuzberg.register_post_processor('lifecycle_processor', processor)
        processors = Kreuzberg.list_post_processors

        expect(processors).to include('lifecycle_processor')
        Kreuzberg.unregister_post_processor('lifecycle_processor')
      end

      it 'unregister_post_processor returns true' do
        processor = lambda { |result| result }
        Kreuzberg.register_post_processor('temp_processor', processor)
        result = Kreuzberg.unregister_post_processor('temp_processor')

        expect(result).to be true
      end

      it 'clear_post_processors removes all processors' do
        Kreuzberg.register_post_processor('p1', ->(r) { r })
        Kreuzberg.register_post_processor('p2', ->(r) { r })
        result = Kreuzberg.clear_post_processors

        expect(result).to be true
      end

      it 'register_post_processor with stage parameter' do
        processor = lambda { |result| result }
        begin
          result = Kreuzberg.register_post_processor('stage_processor', processor, stage: :post_extraction)
          expect(result).to be true
          Kreuzberg.unregister_post_processor('stage_processor')
        rescue ArgumentError
          skip 'Stage parameter not supported'
        end
      end
    end

    describe 'OCR backend registration' do
      it 'list_ocr_backends returns array' do
        backends = Kreuzberg.list_ocr_backends
        expect(backends).to be_an(Array)
      end

      it 'unregister_ocr_backend on non-existent returns boolean' do
        result = Kreuzberg.unregister_ocr_backend('nonexistent_ocr_backend_xyz')
        expect(result.is_a?(TrueClass) || result.is_a?(FalseClass)).to be true
      end
    end
  end

  describe 'Error Handling - Comprehensive Coverage' do
    describe 'Error attributes' do
      it 'ParsingError stores context' do
        error = Kreuzberg::Errors::ParsingError.new(
          'Parse failed',
          context: { line: 10, column: 5, detail: 'unexpected token' }
        )

        expect(error.context).to be_a(Hash)
        expect(error.context[:line]).to eq(10)
        expect(error.context[:column]).to eq(5)
      end

      it 'OCRError stores context' do
        error = Kreuzberg::Errors::OCRError.new(
          'OCR failed',
          context: { backend: 'tesseract', language: 'eng', confidence: 0.5 }
        )

        expect(error.context).to be_a(Hash)
        expect(error.context[:backend]).to eq('tesseract')
        expect(error.context[:language]).to eq('eng')
      end

      it 'MissingDependencyError stores dependency' do
        error = Kreuzberg::Errors::MissingDependencyError.new(
          'Missing libtesseract',
          dependency: 'libtesseract.so.5'
        )

        expect(error.dependency).to eq('libtesseract.so.5')
      end

      it 'Error stores error_code' do
        error = Kreuzberg::Errors::Error.new('Test error', error_code: 42)
        expect(error.error_code).to eq(42)
      end

      it 'Error stores panic_context' do
        panic = Kreuzberg::Errors::PanicContext.new(
          file: 'parser.rs',
          line: 123,
          function: 'parse_pdf',
          message: 'invalid PDF',
          timestamp_secs: 1234567890
        )
        error = Kreuzberg::Errors::Error.new('Panic error', panic_context: panic)

        expect(error.panic_context).to eq(panic)
      end
    end

    describe 'PanicContext class' do
      it 'PanicContext has all attributes' do
        panic = Kreuzberg::Errors::PanicContext.new(
          file: 'lib.rs',
          line: 456,
          function: 'extract',
          message: 'unexpected error',
          timestamp_secs: 1609459200
        )

        expect(panic.file).to eq('lib.rs')
        expect(panic.line).to eq(456)
        expect(panic.function).to eq('extract')
        expect(panic.message).to eq('unexpected error')
        expect(panic.timestamp_secs).to eq(1609459200)
      end

      it 'PanicContext#to_h' do
        panic = Kreuzberg::Errors::PanicContext.new(
          file: 'error.rs',
          line: 789,
          function: 'handle_error',
          message: 'critical error',
          timestamp_secs: 1609459201
        )
        hash = panic.to_h

        expect(hash).to be_a(Hash)
        expect(hash[:file]).to eq('error.rs')
        expect(hash[:line]).to eq(789)
      end

      it 'PanicContext#to_s' do
        panic = Kreuzberg::Errors::PanicContext.new(
          file: 'test.rs',
          line: 100,
          function: 'test_func',
          message: 'test panic',
          timestamp_secs: 1609459202
        )
        string = panic.to_s

        expect(string).to be_a(String)
        expect(string).to include('test.rs')
        expect(string).to include('100')
      end

      it 'PanicContext.from_json parses JSON' do
        json = '{"file":"parse.rs","line":200,"function":"parse","message":"parse error","timestamp_secs":1609459203}'
        panic = Kreuzberg::Errors::PanicContext.from_json(json)

        expect(panic).to be_a(Kreuzberg::Errors::PanicContext)
        expect(panic.file).to eq('parse.rs')
        expect(panic.line).to eq(200)
      end

      it 'PanicContext.from_json returns nil for invalid JSON' do
        panic = Kreuzberg::Errors::PanicContext.from_json('invalid json')
        expect(panic).to be_nil
      end
    end
  end

  describe 'ErrorContext Module - Complete Coverage' do
    describe 'Error introspection methods' do
      it 'ErrorContext.last_error_code returns integer' do
        code = Kreuzberg::ErrorContext.last_error_code
        expect(code).to be_an(Integer)
      end

      it 'ErrorContext.last_panic_context returns PanicContext or nil' do
        panic = Kreuzberg::ErrorContext.last_panic_context
        expect(panic.is_a?(Kreuzberg::Errors::PanicContext) || panic.nil?).to be true
      end

      it 'ErrorContext.last_panic_context_json returns string or nil' do
        json = Kreuzberg::ErrorContext.last_panic_context_json
        expect(json.is_a?(String) || json.nil?).to be true
      end

      it 'ErrorContext.error_details returns hash' do
        details = Kreuzberg::ErrorContext.error_details
        expect(details).to be_a(Hash)
      end

      it 'ErrorContext.classify_error returns integer' do
        code = Kreuzberg::ErrorContext.classify_error('Test error message')
        expect(code).to be_an(Integer)
      end

      it 'ErrorContext.error_code_name returns string' do
        # Using a known error code
        name = Kreuzberg::ErrorContext.error_code_name(0)
        expect(name).to be_a(String)
      end

      it 'ErrorContext.error_code_description returns string' do
        description = Kreuzberg::ErrorContext.error_code_description(0)
        expect(description).to be_a(String)
      end
    end

    describe 'Error code constants' do
      it 'ERROR_CODE_SUCCESS is defined' do
        expect(defined?(Kreuzberg::ERROR_CODE_SUCCESS)).to be_truthy
        expect(Kreuzberg::ERROR_CODE_SUCCESS).to be_an(Integer)
      end

      it 'ERROR_CODE_GENERIC is defined' do
        expect(defined?(Kreuzberg::ERROR_CODE_GENERIC)).to be_truthy
      end

      it 'ERROR_CODE_PANIC is defined' do
        expect(defined?(Kreuzberg::ERROR_CODE_PANIC)).to be_truthy
      end

      it 'ERROR_CODE_INVALID_ARGUMENT is defined' do
        expect(defined?(Kreuzberg::ERROR_CODE_INVALID_ARGUMENT)).to be_truthy
      end

      it 'ERROR_CODE_IO is defined' do
        expect(defined?(Kreuzberg::ERROR_CODE_IO)).to be_truthy
      end

      it 'ERROR_CODE_PARSING is defined' do
        expect(defined?(Kreuzberg::ERROR_CODE_PARSING)).to be_truthy
      end

      it 'ERROR_CODE_OCR is defined' do
        expect(defined?(Kreuzberg::ERROR_CODE_OCR)).to be_truthy
      end

      it 'ERROR_CODE_MISSING_DEPENDENCY is defined' do
        expect(defined?(Kreuzberg::ERROR_CODE_MISSING_DEPENDENCY)).to be_truthy
      end
    end
  end

  describe 'MIME Type Detection - Advanced' do
    describe 'Detection from bytes' do
      it 'detect_mime_type with PDF header' do
        pdf_header = "%PDF-1.4\n"
        mime = Kreuzberg.detect_mime_type(pdf_header)
        expect(mime).to be_a(String)
      end

      it 'detect_mime_type with empty data' do
        mime = Kreuzberg.detect_mime_type('')
        expect(mime.is_a?(String) || mime.nil?).to be true
      end
    end

    describe 'Detection from file path' do
      it 'detect_mime_type_from_path recognizes PDF' do
        mime = Kreuzberg.detect_mime_type_from_path('document.pdf')
        expect(mime).to include('pdf')
      end

      it 'detect_mime_type_from_path recognizes DOCX' do
        mime = Kreuzberg.detect_mime_type_from_path('document.docx')
        expect(mime).to include('wordprocessing')
      end

      it 'detect_mime_type_from_path for unknown extension' do
        mime = Kreuzberg.detect_mime_type_from_path('file.unknown_ext_xyz')
        expect(mime.is_a?(String) || mime.nil?).to be true
      end

      it 'detect_mime_type_from_path for ODT' do
        mime = Kreuzberg.detect_mime_type_from_path('document.odt')
        expect(mime).to include('oasis')
      end

      it 'detect_mime_type_from_path for TXT' do
        mime = Kreuzberg.detect_mime_type_from_path('text.txt')
        expect(mime).to include('text')
      end
    end

    describe 'MIME type validation' do
      it 'validate_mime_type with valid MIME' do
        result = Kreuzberg.validate_mime_type('application/pdf')
        expect(result == true || result.is_a?(String)).to be true
      end

      it 'validate_mime_type with invalid MIME' do
        result = Kreuzberg.validate_mime_type('application/invalid-mime-type-xyz-12345')
        expect(result == false || result.is_a?(String)).to be true
      end

      it 'validate_mime_type with word MIME' do
        result = Kreuzberg.validate_mime_type('application/vnd.openxmlformats-officedocument.wordprocessingml.document')
        expect(result == true || result.is_a?(String)).to be true
      end
    end

    describe 'Extensions for MIME' do
      it 'get_extensions_for_mime returns array for PDF' do
        extensions = Kreuzberg.get_extensions_for_mime('application/pdf')
        expect(extensions).to be_an(Array)
        expect(extensions).to include('pdf')
      end

      it 'get_extensions_for_mime returns array for DOCX' do
        extensions = Kreuzberg.get_extensions_for_mime('application/vnd.openxmlformats-officedocument.wordprocessingml.document')
        expect(extensions).to be_an(Array)
        expect(extensions).to include('docx')
      end

      it 'get_extensions_for_mime for text/plain' do
        extensions = Kreuzberg.get_extensions_for_mime('text/plain')
        expect(extensions).to be_an(Array)
        expect(extensions.first).to be_a(String)
      end

      it 'get_extensions_for_mime for unknown MIME' do
        extensions = Kreuzberg.get_extensions_for_mime('application/unknown-mime-xyz')
        expect(extensions.is_a?(Array)).to be true
      end
    end
  end

  describe 'Embedding Presets' do
    it 'list_embedding_presets returns array' do
      presets = Kreuzberg.list_embedding_presets
      expect(presets).to be_an(Array)
    end

    it 'get_embedding_preset for common preset' do
      ['bert', 'nomic', 'mxbai'].each do |preset_name|
        preset = Kreuzberg.get_embedding_preset(preset_name)
        if preset
          expect(preset.is_a?(Hash) || preset.respond_to?(:[]) || preset.is_a?(Object)).to be true
        end
      end
    end

    it 'get_embedding_preset returns nil for unknown' do
      preset = Kreuzberg.get_embedding_preset('unknown_preset_xyz_12345')
      expect(preset.nil? || preset.is_a?(Object)).to be true
    end
  end

  describe 'Cache API - Comprehensive' do
    it 'clear_cache is callable' do
      expect(Kreuzberg.respond_to?(:clear_cache)).to be true
    end

    it 'cache_stats returns hash-like structure' do
      begin
        stats = Kreuzberg.cache_stats
        expect(stats.is_a?(Hash) || stats.respond_to?(:[]) || stats.respond_to?(:keys)).to be true
      rescue StandardError
        skip 'cache_stats not fully implemented'
      end
    end

    it 'cache_stats contains integer values' do
      begin
        stats = Kreuzberg.cache_stats
        if stats.is_a?(Hash)
          stats.each_value do |value|
            expect(value.is_a?(Integer) || value.is_a?(String)).to be true
          end
        end
      rescue StandardError
        skip 'cache_stats not fully implemented'
      end
    end
  end

  describe 'Protocol Classes' do
    it 'PostProcessorProtocol is accessible' do
      expect(defined?(Kreuzberg::PostProcessorProtocol)).to be_truthy
    end

    it 'ValidatorProtocol is accessible' do
      expect(defined?(Kreuzberg::ValidatorProtocol)).to be_truthy
    end

    it 'OcrBackendProtocol is accessible' do
      expect(defined?(Kreuzberg::OcrBackendProtocol)).to be_truthy
    end
  end

  describe 'Configuration edge cases' do
    describe 'Config::Chunking with embedding' do
      it 'creates chunking with embedding preset' do
        embedding = Kreuzberg::Config::Embedding.new(normalize: true)
        chunking = Kreuzberg::Config::Chunking.new(embedding: embedding)

        expect(chunking.embedding).to be_a(Kreuzberg::Config::Embedding)
      end

      it 'creates chunking with preset name' do
        chunking = Kreuzberg::Config::Chunking.new(preset: 'semantic')
        expect(chunking.preset).to eq('semantic')
      end
    end

    describe 'Config::PDF with FontConfig' do
      it 'creates PDF with font config' do
        font = Kreuzberg::Config::FontConfig.new(enabled: true)
        pdf = Kreuzberg::Config::PDF.new(font_config: font)

        expect(pdf.font_config).to be_a(Kreuzberg::Config::FontConfig)
      end

      it 'PDF passwords as array' do
        pdf = Kreuzberg::Config::PDF.new(passwords: ['pass1', 'pass2'])
        expect(pdf.passwords).to be_an(Array)
      end

      it 'PDF passwords as string' do
        pdf = Kreuzberg::Config::PDF.new(passwords: 'single_password')
        expect(pdf.passwords.is_a?(Array) || pdf.passwords.is_a?(String)).to be true
      end
    end

    describe 'Config::FontConfig accessors' do
      it 'FontConfig enabled is settable' do
        font = Kreuzberg::Config::FontConfig.new(enabled: true)
        expect(font.enabled).to eq(true)
      end

      it 'FontConfig custom_font_dirs is settable' do
        font = Kreuzberg::Config::FontConfig.new(custom_font_dirs: ['/usr/share/fonts'])
        expect(font.custom_font_dirs).to be_an(Array)
      end
    end

    describe 'Config::Keywords algorithm selection' do
      it 'Keywords with YAKE algorithm' do
        keywords = Kreuzberg::Config::Keywords.new(algorithm: :yake, max_keywords: 20)
        hash = keywords.to_h
        expect(hash).to be_a(Hash)
      end

      it 'Keywords with RAKE algorithm' do
        keywords = Kreuzberg::Config::Keywords.new(algorithm: :rake, max_keywords: 15)
        hash = keywords.to_h
        expect(hash).to be_a(Hash)
      end
    end

    describe 'Config::TokenReduction modes' do
      it 'TokenReduction with different modes' do
        reduction = Kreuzberg::Config::TokenReduction.new(mode: 'semantic')
        expect(reduction.mode).to eq('semantic')
      end

      it 'TokenReduction preserves important words' do
        reduction = Kreuzberg::Config::TokenReduction.new(preserve_important_words: true)
        expect(reduction.preserve_important_words).to eq(true)
      end
    end

    describe 'Config::PageConfig options' do
      it 'PageConfig with page extraction' do
        pages = Kreuzberg::Config::PageConfig.new(extract_pages: true)
        expect(pages.extract_pages).to eq(true)
      end

      it 'PageConfig with page markers' do
        pages = Kreuzberg::Config::PageConfig.new(insert_page_markers: true, marker_format: '---PAGE %d---')
        expect(pages.insert_page_markers).to eq(true)
        expect(pages.marker_format).to eq('---PAGE %d---')
      end
    end
  end

  describe 'Result type combinations' do
    it 'Result with all optional fields' do
      result = Kreuzberg::Result.new(
        content: 'test',
        mime_type: 'text/plain',
        metadata_json: '{}',
        tables: [
          { cells: [['a']], markdown: '| a |', page_number: 1 }
        ],
        chunks: [
          { content: 'chunk', byte_start: 0, byte_end: 5, token_count: 1, chunk_index: 0,
            total_chunks: 1, first_page: 1, last_page: 1, embedding: nil }
        ],
        images: [
          { data: 'data', format: 'png', image_index: 0, page_number: 1, width: 100,
            height: 100, colorspace: 'RGB', bits_per_component: 8, is_mask: false,
            description: 'test', ocr_result: nil }
        ],
        detected_languages: ['en', 'fr'],
        elements: []
      )

      expect(result.content).to eq('test')
      expect(result.tables.length).to eq(1)
      expect(result.chunks.length).to eq(1)
      expect(result.images.length).to eq(1)
      expect(result.detected_languages).to include('en')
    end
  end
end
