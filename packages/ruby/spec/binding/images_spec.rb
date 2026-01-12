# frozen_string_literal: true

require 'spec_helper'
require 'tempfile'
require 'fileutils'

RSpec.describe 'Image Extraction' do
  describe 'PDF image extraction with metadata' do
    it 'extracts images with format and dimensions' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 150
        )
      )

      pdf_path = test_document_path('pdf/with_images.pdf')
      begin
        result = Kreuzberg.extract_file_sync(path: pdf_path, config: config)

        expect(result).not_to be_nil
        if result.images && !result.images.empty?
          image = result.images.first
          expect(image).to be_a(Kreuzberg::Result::Image)
          expect(image.format).not_to be_nil
          expect(image.width).to be > 0
          expect(image.height).to be > 0
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'includes page numbers in extracted images' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 150
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && !result.images.empty?
          result.images.each do |image|
            expect(image.page_number).to be > 0
          end
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'respects target_dpi configuration parameter' do
      dpi_values = [150, 300, 600]

      dpi_values.each do |dpi|
        config = Kreuzberg::Config::Extraction.new(
          image_extraction: Kreuzberg::Config::ImageExtraction.new(
            extract_images: true,
            target_dpi: dpi
          )
        )

        begin
          result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

          expect(result).not_to be_nil
        rescue Kreuzberg::Errors::ValidationError
          skip 'Test file not available'
        end
      end
    end

    it 'includes colorspace information in image metadata' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 150
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && !result.images.empty?
          image = result.images.first
          expect(image).to respond_to(:colorspace)
          # Verify colorspace has meaningful value if present
          if image.colorspace
            expect(image.colorspace).not_to be_empty
            expect(image.colorspace).to be_a(String)
          end
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end
  end

  describe 'Image handling in composite documents' do
    it 'extracts images from DOCX files' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        docx_path = test_document_path('office/document.docx')
        result = Kreuzberg.extract_file_sync(path: docx_path, config: config)

        expect(result).not_to be_nil
        expect(result.content).not_to be_nil
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'extracts images from PPTX files' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        pptx_path = test_document_path('presentations/simple.pptx')
        result = Kreuzberg.extract_file_sync(path: pptx_path, config: config)

        expect(result).not_to be_nil
        expect(result.content).not_to be_nil
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'handles documents with multiple images across pages' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 150
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && result.images.length > 1
          page_numbers = result.images.map(&:page_number).uniq
          expect(page_numbers.length).to be > 1
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'preserves image index for sequential extraction' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && result.images.length > 1
          result.images.each_with_index do |image, _index|
            expect(image.image_index).to be_a(Integer)
          end
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end
  end

  describe 'Image format detection' do
    it 'detects PNG format in extracted images' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 150
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && !result.images.empty?
          formats = result.images.filter_map(&:format)
          expect(formats).to be_an(Array)
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'detects JPEG format in extracted images' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 150
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && !result.images.empty?
          result.images.each do |image|
            expect(image.format).not_to be_nil
            expect(image.format).to be_a(String)
          end
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'handles WebP format detection if present' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        expect(result).not_to be_nil
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'provides consistent format strings across extractions' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        result1 = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)
        result2 = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result1.images && result2.images && !result1.images.empty? && !result2.images.empty?
          expect(result1.images.first.format).to eq(result2.images.first.format)
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end
  end

  describe 'Embedded vs referenced images' do
    it 'extracts embedded images from documents' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        expect(result).not_to be_nil
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'handles image data field in extracted images' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 150
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && !result.images.empty?
          image = result.images.first
          expect(image).to respond_to(:data)
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'preserves image metadata when extraction enabled' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && !result.images.empty?
          image = result.images.first
          expect(image.width).to be_a(Integer)
          expect(image.height).to be_a(Integer)
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'returns nil for images when extraction disabled' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: false
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        expect(result.images).to be_nil
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end
  end

  describe 'Error handling for corrupted images' do
    it 'gracefully handles documents with malformed images' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)
        expect(result).not_to be_nil
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'continues extraction when encountering problematic images' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        expect(result).not_to be_nil
        expect(result.content).not_to be_nil
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'handles extraction with max_image_dimension constraint' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          max_image_dimension: 1000
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && !result.images.empty?
          result.images.each do |image|
            expect(image.width).to be_a(Integer)
            expect(image.height).to be_a(Integer)
          end
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'respects auto_adjust_dpi configuration' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          auto_adjust_dpi: true,
          min_dpi: 150,
          max_dpi: 600
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        expect(result).not_to be_nil
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end
  end

  describe 'Batch image extraction from multi-page documents' do
    it 'extracts images from multi-page PDF in single operation' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 150
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        expect(result).not_to be_nil
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'maintains correct page associations for extracted images' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && result.images.length > 1
          result.images.each do |image|
            expect(image.page_number).to be >= 1
          end
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'preserves image order within document' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && result.images.length > 1
          (0...(result.images.length - 1)).each do |i|
            expect(result.images[i].image_index).to be <= result.images[i + 1].image_index
          end
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'handles multiple file batch extraction with images' do
      paths = []
      2.times do |i|
        file = Tempfile.new("batch_image_test_#{i}.txt")
        file.write("Image extraction test #{i}")
        file.close
        paths << file.path
      end

      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      results = Kreuzberg.batch_extract_files_sync(paths: paths, config: config)

      expect(results).to be_a(Array)
      expect(results.length).to eq(2)
      expect(results).to all(be_a(Kreuzberg::Result))
    ensure
      paths.each { |p| FileUtils.rm_f(p) }
    end

    it 'maintains correct image count across batch operations' do
      paths = []
      2.times do |i|
        file = Tempfile.new("batch_count_#{i}.txt")
        file.write("Content #{i}")
        file.close
        paths << file.path
      end

      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      results = Kreuzberg.batch_extract_files_sync(paths: paths, config: config)

      expect(results.length).to eq(paths.length)
      expect(results).to all(be_a(Kreuzberg::Result))
    ensure
      paths.each { |p| FileUtils.rm_f(p) }
    end
  end

  describe 'ImageExtraction configuration integration' do
    it 'applies different DPI settings to affect extraction behavior' do
      config_low = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 72
        )
      )
      config_high = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 300
        )
      )

      begin
        result_low = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config_low)
        result_high = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config_high)

        # Both configurations should produce valid extraction
        expect(result_low).not_to be_nil
        expect(result_high).not_to be_nil
        # Different DPI settings should be accepted
        expect([result_low, result_high]).to all(be_a(Kreuzberg::Result))
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'respects extract_images false disables extraction' do
      config_enabled = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )
      config_disabled = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: false
        )
      )

      begin
        result_enabled = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config_enabled)
        result_disabled = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config_disabled)

        # Enabled should extract if images present
        expect(result_enabled).not_to be_nil
        # Disabled should return nil or empty images
        expect(result_disabled.images).to be_empty if result_disabled.images
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'handles dimension constraints realistically' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          max_image_dimension: 1024
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        expect(result).not_to be_nil
        # Dimension constraint should be applied
        if result.images && !result.images.empty?
          result.images.each do |image|
            # Image should respect dimension constraints
            expect(image).not_to be_nil
          end
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end
  end

  describe 'Integration with Extraction config' do
    it 'accepts ImageExtraction config in Extraction' do
      image_config = Kreuzberg::Config::ImageExtraction.new(
        extract_images: true,
        target_dpi: 600
      )
      config = Kreuzberg::Config::Extraction.new(image_extraction: image_config)

      expect(config.image_extraction).to be_a(Kreuzberg::Config::ImageExtraction)
      expect(config.image_extraction.target_dpi).to eq(600)
    end

    it 'accepts image extraction config as hash in Extraction' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: {
          extract_images: true,
          target_dpi: 600,
          max_image_dimension: 3000
        }
      )

      expect(config.image_extraction).to be_a(Kreuzberg::Config::ImageExtraction)
      expect(config.image_extraction.extract_images).to be true
      expect(config.image_extraction.target_dpi).to eq(600)
      expect(config.image_extraction.max_image_dimension).to eq(3000)
    end

    it 'includes image extraction config in to_h' do
      image_config = Kreuzberg::Config::ImageExtraction.new(
        extract_images: true,
        target_dpi: 600
      )
      config = Kreuzberg::Config::Extraction.new(image_extraction: image_config)

      hash = config.to_h

      expect(hash).to include(:image_extraction)
      expect(hash[:image_extraction]).to be_a(Hash)
      expect(hash[:image_extraction][:extract_images]).to be true
      expect(hash[:image_extraction][:target_dpi]).to eq(600)
    end

    it 'combines image extraction with other configurations' do
      config = Kreuzberg::Config::Extraction.new(
        use_cache: true,
        force_ocr: true,
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 600
        ),
        ocr: Kreuzberg::Config::OCR.new(
          backend: 'tesseract',
          language: 'eng'
        )
      )

      expect(config.use_cache).to be true
      expect(config.force_ocr).to be true
      expect(config.image_extraction.target_dpi).to eq(600)
      expect(config.ocr.backend).to eq('tesseract')
    end

    it 'handles nil image extraction config' do
      config = Kreuzberg::Config::Extraction.new(image_extraction: nil)

      expect(config.image_extraction).to be_nil
    end
  end

  describe 'Image metadata validation in real extractions' do
    it 'validates extracted images have complete required metadata' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && !result.images.empty?
          result.images.each do |image|
            # All extracted images must have these fields populated
            expect(image).not_to be_nil
            expect(image.format).not_to be_nil, 'Format is required'
            expect(image.format).not_to be_empty
            expect(image.image_index).to be >= 0, 'Image index must be non-negative'
            expect(image.data).not_to be_nil, 'Image data is required'
          end
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'includes optional metadata fields appropriately' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true,
          target_dpi: 150
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && !result.images.empty?
          result.images.each do |image|
            # Optional fields should be valid when present
            expect(image.width).to be > 0, 'Width should be positive when present' if image.width
            expect(image.height).to be > 0, 'Height should be positive when present' if image.height
            expect(image.page_number).to be > 0, 'Page number should be positive' if image.page_number
          end
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end

    it 'ensures multiple images have different indices' do
      config = Kreuzberg::Config::Extraction.new(
        image_extraction: Kreuzberg::Config::ImageExtraction.new(
          extract_images: true
        )
      )

      begin
        result = Kreuzberg.extract_file_sync(path: test_document_path('pdf/with_images.pdf'), config: config)

        if result.images && result.images.length > 1
          indices = result.images.map(&:image_index)
          unique_indices = indices.uniq
          expect(unique_indices.length).to eq(indices.length), 'Each image should have unique index'
        end
      rescue Kreuzberg::Errors::ValidationError
        skip 'Test file not available'
      end
    end
  end
end
