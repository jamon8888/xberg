# frozen_string_literal: true

begin
  require 'json'
rescue LoadError
  require 'json/pure'
end

module Kreuzberg
  # @example
  # rubocop:disable Metrics/ClassLength
  class Result
    attr_reader :content, :mime_type, :metadata, :metadata_json, :tables,
                :detected_languages, :chunks, :images, :pages, :elements, :djot_content

    # @!attribute [r] cells
    #   @return [Array<Array<String>>] Table cells (2D array)
    # @!attribute [r] markdown
    #   @return [String] Markdown representation
    # @!attribute [r] page_number
    #   @return [Integer] Page number where table was found
    Table = Struct.new(:cells, :markdown, :page_number, keyword_init: true) do
      def to_h
        { cells: cells, markdown: markdown, page_number: page_number }
      end
    end

    # @!attribute [r] content
    #   @return [String] Chunk content
    # @!attribute [r] byte_start
    #   @return [Integer] Starting byte offset (UTF-8)
    # @!attribute [r] byte_end
    #   @return [Integer] Ending byte offset (UTF-8)
    # @!attribute [r] token_count
    #   @return [Integer, nil] Approximate token count (may be nil)
    # @!attribute [r] first_page
    #   @return [Integer, nil] First page number (1-indexed)
    # @!attribute [r] last_page
    #   @return [Integer, nil] Last page number (1-indexed)
    Chunk = Struct.new(
      :content,
      :byte_start,
      :byte_end,
      :token_count,
      :chunk_index,
      :total_chunks,
      :first_page,
      :last_page,
      :embedding,
      keyword_init: true
    ) do
      def to_h
        {
          content: content,
          byte_start: byte_start,
          byte_end: byte_end,
          token_count: token_count,
          chunk_index: chunk_index,
          total_chunks: total_chunks,
          first_page: first_page,
          last_page: last_page,
          embedding: embedding
        }
      end
    end

    Image = Struct.new(
      :data,
      :format,
      :image_index,
      :page_number,
      :width,
      :height,
      :colorspace,
      :bits_per_component,
      :is_mask,
      :description,
      :ocr_result,
      keyword_init: true
    ) do
      def to_h
        {
          data: data,
          format: format,
          image_index: image_index,
          page_number: page_number,
          width: width,
          height: height,
          colorspace: colorspace,
          bits_per_component: bits_per_component,
          is_mask: is_mask,
          description: description,
          ocr_result: ocr_result&.to_h
        }
      end
    end

    # @!attribute [r] page_number
    #   @return [Integer] Page number (1-indexed)
    # @!attribute [r] content
    #   @return [String] Text content for this page
    # @!attribute [r] tables
    #   @return [Array<Table>] Tables on this page
    # @!attribute [r] images
    #   @return [Array<Image>] Images on this page
    PageContent = Struct.new(:page_number, :content, :tables, :images, keyword_init: true) do
      def to_h
        {
          page_number: page_number,
          content: content,
          tables: tables.map(&:to_h),
          images: images.map(&:to_h)
        }
      end
    end

    # @!attribute [r] x0
    #   @return [Float] Left x-coordinate
    # @!attribute [r] y0
    #   @return [Float] Bottom y-coordinate
    # @!attribute [r] x1
    #   @return [Float] Right x-coordinate
    # @!attribute [r] y1
    #   @return [Float] Top y-coordinate
    ElementBoundingBox = Struct.new(:x0, :y0, :x1, :y1, keyword_init: true) do
      def to_h
        { x0: x0, y0: y0, x1: x1, y1: y1 }
      end
    end

    # @!attribute [r] page_number
    #   @return [Integer, nil] Page number (1-indexed)
    # @!attribute [r] filename
    #   @return [String, nil] Source filename or document name
    # @!attribute [r] coordinates
    #   @return [ElementBoundingBox, nil] Bounding box coordinates if available
    # @!attribute [r] element_index
    #   @return [Integer, nil] Position index in the element sequence
    # @!attribute [r] additional
    #   @return [Hash<String, String>] Additional custom metadata
    ElementMetadataStruct = Struct.new(
      :page_number,
      :filename,
      :coordinates,
      :element_index,
      :additional,
      keyword_init: true
    ) do
      def to_h
        {
          page_number: page_number,
          filename: filename,
          coordinates: coordinates&.to_h,
          element_index: element_index,
          additional: additional
        }
      end
    end

    # @!attribute [r] element_id
    #   @return [String] Unique element identifier
    # @!attribute [r] element_type
    #   @return [String] Semantic type of the element
    # @!attribute [r] text
    #   @return [String] Text content of the element
    # @!attribute [r] metadata
    #   @return [ElementMetadataStruct] Metadata about the element
    ElementStruct = Struct.new(:element_id, :element_type, :text, :metadata, keyword_init: true) do
      def to_h
        {
          element_id: element_id,
          element_type: element_type,
          text: text,
          metadata: metadata&.to_h
        }
      end
    end

    # Initialize from native hash result
    #
    # @param hash [Hash] Hash returned from native extension
    #
    # rubocop:disable Metrics/AbcSize
    def initialize(hash)
      @content = get_value(hash, 'content', '')
      @mime_type = get_value(hash, 'mime_type', '')
      @metadata_json = get_value(hash, 'metadata_json', '{}')
      @metadata = parse_metadata(@metadata_json)
      @tables = parse_tables(get_value(hash, 'tables'))
      @detected_languages = parse_detected_languages(get_value(hash, 'detected_languages'))
      @chunks = parse_chunks(get_value(hash, 'chunks'))
      @images = parse_images(get_value(hash, 'images'))
      @pages = parse_pages(get_value(hash, 'pages'))
      @elements = parse_elements(get_value(hash, 'elements'))
      @djot_content = parse_djot_content(get_value(hash, 'djot_content'))
    end
    # rubocop:enable Metrics/AbcSize

    # Convert to hash
    #
    # @return [Hash] Hash representation
    #
    def to_h
      {
        content: @content,
        mime_type: @mime_type,
        metadata: @metadata,
        tables: serialize_tables,
        detected_languages: @detected_languages,
        chunks: serialize_chunks,
        images: serialize_images,
        pages: serialize_pages,
        elements: serialize_elements,
        djot_content: @djot_content&.to_h
      }
    end

    # Convert to JSON
    #
    # @return [String] JSON representation
    #
    def to_json(*)
      to_h.to_json(*)
    end

    # Get the total number of pages in the document
    #
    # @return [Integer] Total page count (>= 0), or -1 on error
    #
    # @example
    #   result = Kreuzberg.extract_file_sync("document.pdf")
    #   puts "Document has #{result.page_count} pages"
    #
    def page_count
      if @metadata.is_a?(Hash) && @metadata['pages'].is_a?(Hash)
        @metadata['pages']['total_count'] || 0
      else
        0
      end
    end

    # Get the total number of text chunks
    #
    # Returns 0 if chunking was not performed.
    #
    # @return [Integer] Total chunk count (>= 0), or -1 on error
    #
    # @example
    #   result = Kreuzberg.extract_file_sync("document.pdf")
    #   puts "Document has #{result.chunk_count} chunks"
    #
    def chunk_count
      @chunks&.length || 0
    end

    # Get the primary detected language
    #
    # @return [String, nil] ISO 639 language code (e.g., "en", "de"), or nil if not detected
    #
    # @example
    #   result = Kreuzberg.extract_file_sync("document.pdf")
    #   lang = result.detected_language
    #   puts "Language: #{lang}" if lang
    #
    def detected_language
      return @metadata['language'] if @metadata.is_a?(Hash) && @metadata['language']
      return @detected_languages&.first if @detected_languages&.any?

      nil
    end

    # Get a metadata field by name
    #
    # Supports dot notation for nested fields (e.g., "format.pages").
    #
    # @param name [String, Symbol] Field name
    # @return [Object, nil] Field value, or nil if field doesn't exist
    #
    # @example Get a top-level field
    #   result = Kreuzberg.extract_file_sync("document.pdf")
    #   title = result.metadata_field("title")
    #   puts "Title: #{title}" if title
    #
    # @example Get a nested field
    #   format_info = result.metadata_field("format.pages")
    #
    def metadata_field(name)
      return nil unless @metadata.is_a?(Hash)

      parts = name.to_s.split('.')
      value = @metadata

      parts.each do |part|
        return nil unless value.is_a?(Hash)

        value = value[part]
      end

      value
    end

    private

    def serialize_tables
      @tables.map(&:to_h)
    end

    def serialize_chunks
      @chunks&.map(&:to_h)
    end

    def serialize_images
      @images&.map(&:to_h)
    end

    def serialize_pages
      @pages&.map(&:to_h)
    end

    def serialize_elements
      @elements&.map(&:to_h)
    end

    def get_value(hash, key, default = nil)
      hash[key] || hash[key.to_sym] || default
    end

    def parse_metadata(metadata_json)
      JSON.parse(metadata_json)
    rescue JSON::ParserError
      {}
    end

    def parse_tables(tables_data)
      return [] if tables_data.nil? || tables_data.empty?

      tables_data.map do |table_hash|
        Table.new(
          cells: table_hash['cells'] || [],
          markdown: table_hash['markdown'] || '',
          page_number: table_hash['page_number'] || 0
        )
      end
    end

    def parse_detected_languages(langs_data)
      return nil if langs_data.nil?

      langs_data.is_a?(Array) ? langs_data : []
    end

    def parse_chunks(chunks_data)
      return [] if chunks_data.nil? || chunks_data.empty?

      chunks_data.map do |chunk_hash|
        Chunk.new(
          content: chunk_hash['content'],
          byte_start: chunk_hash['byte_start'],
          byte_end: chunk_hash['byte_end'],
          token_count: chunk_hash['token_count'],
          chunk_index: chunk_hash['chunk_index'],
          total_chunks: chunk_hash['total_chunks'],
          first_page: chunk_hash['first_page'],
          last_page: chunk_hash['last_page'],
          embedding: chunk_hash['embedding']
        )
      end
    end

    def parse_images(images_data)
      return nil if images_data.nil?

      images_data.map do |image_hash|
        data = image_hash['data']
        data = data.dup.force_encoding(Encoding::BINARY) if data.respond_to?(:force_encoding)
        Image.new(
          data: data,
          format: image_hash['format'],
          image_index: image_hash['image_index'],
          page_number: image_hash['page_number'],
          width: image_hash['width'],
          height: image_hash['height'],
          colorspace: image_hash['colorspace'],
          bits_per_component: image_hash['bits_per_component'],
          is_mask: image_hash['is_mask'],
          description: image_hash['description'],
          ocr_result: image_hash['ocr_result'] ? Result.new(image_hash['ocr_result']) : nil
        )
      end
    end

    def parse_pages(pages_data)
      return nil if pages_data.nil?

      pages_data.map do |page_hash|
        PageContent.new(
          page_number: page_hash['page_number'],
          content: page_hash['content'],
          tables: parse_tables(page_hash['tables']),
          images: parse_images(page_hash['images'])
        )
      end
    end

    def parse_elements(elements_data)
      return nil if elements_data.nil?

      elements_data.map { |element_hash| parse_element(element_hash) }
    end

    def parse_element(element_hash)
      metadata_hash = element_hash['metadata'] || {}
      coordinates = parse_element_coordinates(metadata_hash['coordinates'])

      metadata = ElementMetadataStruct.new(
        page_number: metadata_hash['page_number'],
        filename: metadata_hash['filename'],
        coordinates: coordinates,
        element_index: metadata_hash['element_index'],
        additional: metadata_hash['additional'] || {}
      )

      ElementStruct.new(
        element_id: element_hash['element_id'],
        element_type: element_hash['element_type'],
        text: element_hash['text'],
        metadata: metadata
      )
    end

    def parse_element_coordinates(coordinates_data)
      return nil if coordinates_data.nil?

      ElementBoundingBox.new(
        x0: coordinates_data['x0'].to_f,
        y0: coordinates_data['y0'].to_f,
        x1: coordinates_data['x1'].to_f,
        y1: coordinates_data['y1'].to_f
      )
    end

    def parse_djot_content(djot_data)
      return nil if djot_data.nil?

      DjotContent.new(djot_data)
    end
  end
  # rubocop:enable Metrics/ClassLength
end
