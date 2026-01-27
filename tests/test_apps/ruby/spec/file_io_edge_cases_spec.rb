# frozen_string_literal: true

require_relative 'spec_helper'

RSpec.describe 'Kreuzberg File I/O Edge Cases' do
  describe 'Non-existent files' do
    it 'extract_file_sync raises IOError for missing file' do
      expect do
        Kreuzberg.extract_file_sync(path: '/nonexistent/path/to/file.pdf')
      end.to raise_error(Kreuzberg::Errors::IOError)
    end

    it 'extract_file raises IOError for missing file' do
      expect do
        Kreuzberg.extract_file(path: '/nonexistent/file_async.pdf')
      end.to raise_error(Kreuzberg::Errors::IOError)
    end

    it 'batch_extract_files_sync handles missing files' do
      paths = [
        '/nonexistent/file1.pdf',
        test_document_path('documents/fake.docx')
      ]
      expect do
        Kreuzberg.batch_extract_files_sync(paths: paths)
      end.to raise_error(Kreuzberg::Errors::IOError)
    end
  end

  describe 'Path handling' do
    it 'extract_file_sync accepts string path' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to be_a(Kreuzberg::Result)
    end

    it 'extract_file_sync accepts Pathname' do
      path = Pathname.new(test_document_path('documents/fake.docx'))
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to be_a(Kreuzberg::Result)
    end

    it 'batch_extract_files_sync accepts string paths' do
      paths = [
        test_document_path('documents/fake.docx'),
        test_document_path('documents/simple.odt')
      ]
      results = Kreuzberg.batch_extract_files_sync(paths: paths)

      expect(results).to be_an(Array)
      expect(results.length).to eq(2)
    end

    it 'batch_extract_files_sync accepts Pathname objects' do
      paths = [
        Pathname.new(test_document_path('documents/fake.docx')),
        Pathname.new(test_document_path('documents/simple.odt'))
      ]
      results = Kreuzberg.batch_extract_files_sync(paths: paths)

      expect(results).to be_an(Array)
      expect(results.length).to eq(2)
    end

    it 'batch_extract_files_sync accepts mixed string and Pathname' do
      paths = [
        test_document_path('documents/fake.docx'),
        Pathname.new(test_document_path('documents/simple.odt'))
      ]
      results = Kreuzberg.batch_extract_files_sync(paths: paths)

      expect(results).to be_an(Array)
      expect(results.length).to eq(2)
    end
  end

  describe 'MIME type handling' do
    it 'extract_file_sync auto-detects MIME type' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result.mime_type).to include('wordprocessing')
    end

    it 'extract_file_sync uses provided MIME type' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(
        path: path,
        mime_type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
      )

      expect(result.mime_type).to include('wordprocessing')
    end

    it 'extract_bytes_sync requires MIME type' do
      data = read_test_document('documents/fake.docx')
      expect do
        Kreuzberg.extract_bytes_sync(
          data: data,
          mime_type: nil
        )
      end.to raise_error(TypeError)
    end

    it 'extract_bytes_sync with valid MIME type' do
      data = read_test_document('documents/fake.docx')
      result = Kreuzberg.extract_bytes_sync(
        data: data,
        mime_type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
      )

      expect(result).to be_a(Kreuzberg::Result)
    end
  end

  describe 'Data validation' do
    it 'extract_bytes_sync with empty data raises error' do
      expect do
        Kreuzberg.extract_bytes_sync(
          data: '',
          mime_type: 'application/pdf'
        )
      end.to raise_error(Kreuzberg::Errors::ParsingError)
    end

    it 'batch_extract_bytes_sync with mismatched array lengths' do
      data = [
        read_test_document('documents/fake.docx'),
        read_test_document('documents/simple.odt')
      ]
      mime_types = ['application/vnd.openxmlformats-officedocument.wordprocessingml.document']

      expect do
        Kreuzberg.batch_extract_bytes_sync(data_array: data, mime_types: mime_types)
      end.to raise_error
    end

    it 'batch_extract_bytes_sync with matching arrays succeeds' do
      data = [
        read_test_document('documents/fake.docx'),
        read_test_document('documents/simple.odt')
      ]
      mime_types = [
        'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        'application/vnd.oasis.opendocument.text'
      ]
      results = Kreuzberg.batch_extract_bytes_sync(data_array: data, mime_types: mime_types)

      expect(results).to be_an(Array)
      expect(results.length).to eq(2)
    end
  end

  describe 'Config validation' do
    it 'extract_file_sync accepts Config object' do
      path = test_document_path('documents/fake.docx')
      config = Kreuzberg::Config::Extraction.new(output_format: 'markdown')
      result = Kreuzberg.extract_file_sync(path: path, config: config)

      expect(result).to be_a(Kreuzberg::Result)
    end

    it 'extract_file_sync accepts config hash' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path, config: { output_format: 'plain' })

      expect(result).to be_a(Kreuzberg::Result)
    end

    it 'extract_file_sync accepts nil config' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path, config: nil)

      expect(result).to be_a(Kreuzberg::Result)
    end

    it 'batch_extract_files_sync accepts config' do
      paths = [test_document_path('documents/fake.docx')]
      config = Kreuzberg::Config::Extraction.new(force_ocr: false)
      results = Kreuzberg.batch_extract_files_sync(paths: paths, config: config)

      expect(results).to be_an(Array)
    end
  end

  describe 'Result consistency' do
    it 'repeated extractions produce same content' do
      path = test_document_path('documents/fake.docx')
      result1 = Kreuzberg.extract_file_sync(path: path)
      result2 = Kreuzberg.extract_file_sync(path: path)

      expect(result1.content).to eq(result2.content)
      expect(result1.mime_type).to eq(result2.mime_type)
    end

    it 'sync and async extractions match' do
      path = test_document_path('documents/fake.docx')
      sync_result = Kreuzberg.extract_file_sync(path: path)
      async_result = Kreuzberg.extract_file(path: path)

      expect(sync_result.content).to eq(async_result.content)
    end

    it 'file and bytes extraction match' do
      path = test_document_path('documents/fake.docx')
      file_result = Kreuzberg.extract_file_sync(path: path)

      data = read_test_document('documents/fake.docx')
      bytes_result = Kreuzberg.extract_bytes_sync(
        data: data,
        mime_type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
      )

      expect(file_result.content).to eq(bytes_result.content)
    end

    it 'batch and individual extraction match' do
      path = test_document_path('documents/fake.docx')
      individual_result = Kreuzberg.extract_file_sync(path: path)
      batch_results = Kreuzberg.batch_extract_files_sync(paths: [path])

      expect(individual_result.content).to eq(batch_results[0].content)
    end
  end

  describe 'Result attributes completeness' do
    it 'all result attributes are present' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result).to respond_to(:content)
      expect(result).to respond_to(:mime_type)
      expect(result).to respond_to(:metadata)
      expect(result).to respond_to(:metadata_json)
      expect(result).to respond_to(:tables)
      expect(result).to respond_to(:chunks)
      expect(result).to respond_to(:images)
      expect(result).to respond_to(:detected_languages)
      expect(result).to respond_to(:djot_content)
    end

    it 'result content is non-empty' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result.content).to be_a(String)
      expect(result.content).not_to be_empty
    end

    it 'result mime_type is non-empty' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result.mime_type).to be_a(String)
      expect(result.mime_type).not_to be_empty
    end

    it 'result metadata is Hash-like' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result.metadata.is_a?(Hash) || result.metadata.respond_to?(:[]) || result.metadata.nil?).to be true
    end

    it 'result tables is Array' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result.tables.is_a?(Array) || result.tables.nil?).to be true
    end

    it 'result chunks is Array or nil' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result.chunks.is_a?(Array) || result.chunks.nil?).to be true
    end

    it 'result images is Array or nil' do
      path = test_document_path('documents/fake.docx')
      result = Kreuzberg.extract_file_sync(path: path)

      expect(result.images.is_a?(Array) || result.images.nil?).to be true
    end
  end

  describe 'Config state management' do
    it 'config changes do not affect previous results' do
      config = Kreuzberg::Config::Extraction.new(output_format: 'plain')
      path = test_document_path('documents/fake.docx')

      result1 = Kreuzberg.extract_file_sync(path: path, config: config)
      config.output_format = 'markdown'
      result2 = Kreuzberg.extract_file_sync(path: path, config: config)

      expect(result1).to be_a(Kreuzberg::Result)
      expect(result2).to be_a(Kreuzberg::Result)
    end

    it 'multiple config objects are independent' do
      config1 = Kreuzberg::Config::Extraction.new(output_format: 'plain')
      config2 = Kreuzberg::Config::Extraction.new(output_format: 'markdown')

      expect(config1.output_format).to eq('plain')
      expect(config2.output_format).to eq('markdown')
      expect(config1.output_format).not_to eq(config2.output_format)
    end

    it 'config object can be reused for multiple extractions' do
      config = Kreuzberg::Config::Extraction.new(output_format: 'plain')
      paths = [
        test_document_path('documents/fake.docx'),
        test_document_path('documents/simple.odt')
      ]

      result1 = Kreuzberg.extract_file_sync(path: paths[0], config: config)
      result2 = Kreuzberg.extract_file_sync(path: paths[1], config: config)

      expect(result1).to be_a(Kreuzberg::Result)
      expect(result2).to be_a(Kreuzberg::Result)
      expect(config.output_format).to eq('plain')
    end
  end
end
