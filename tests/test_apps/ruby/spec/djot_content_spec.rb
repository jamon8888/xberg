# frozen_string_literal: true

require_relative 'spec_helper'

RSpec.describe 'Kreuzberg Djot Content - Nested Types' do
  describe 'DjotContent class' do
    it 'initializes from hash' do
      djot_hash = {
        plain_text: 'test text',
        blocks: [],
        metadata_json: '{}',
        tables: [],
        images: [],
        links: [],
        footnotes: [],
        attributes: {}
      }
      djot = Kreuzberg::Result::DjotContent.new(djot_hash)

      expect(djot).to be_a(Kreuzberg::Result::DjotContent)
      expect(djot.plain_text).to eq('test text')
    end

    it 'to_h produces hash' do
      djot_hash = {
        plain_text: 'content',
        blocks: [],
        metadata_json: '{}',
        tables: [],
        images: [],
        links: [],
        footnotes: []
      }
      djot = Kreuzberg::Result::DjotContent.new(djot_hash)
      hash = djot.to_h

      expect(hash).to be_a(Hash)
      expect(hash[:plain_text]).to eq('content')
    end

    it 'accesses blocks array' do
      djot_hash = {
        plain_text: 'text',
        blocks: [],
        metadata_json: '{}',
        tables: [],
        images: [],
        links: [],
        footnotes: []
      }
      djot = Kreuzberg::Result::DjotContent.new(djot_hash)

      expect(djot.blocks).to be_an(Array)
    end

    it 'accesses images array' do
      djot_hash = {
        plain_text: 'text',
        blocks: [],
        metadata_json: '{}',
        tables: [],
        images: [],
        links: [],
        footnotes: []
      }
      djot = Kreuzberg::Result::DjotContent.new(djot_hash)

      expect(djot.images).to be_an(Array)
    end

    it 'accesses links array' do
      djot_hash = {
        plain_text: 'text',
        blocks: [],
        metadata_json: '{}',
        tables: [],
        images: [],
        links: [],
        footnotes: []
      }
      djot = Kreuzberg::Result::DjotContent.new(djot_hash)

      expect(djot.links).to be_an(Array)
    end

    it 'accesses footnotes array' do
      djot_hash = {
        plain_text: 'text',
        blocks: [],
        metadata_json: '{}',
        tables: [],
        images: [],
        links: [],
        footnotes: []
      }
      djot = Kreuzberg::Result::DjotContent.new(djot_hash)

      expect(djot.footnotes).to be_an(Array)
    end

    it 'accesses metadata hash' do
      djot_hash = {
        plain_text: 'text',
        blocks: [],
        metadata_json: '{"author": "test"}',
        tables: [],
        images: [],
        links: [],
        footnotes: []
      }
      djot = Kreuzberg::Result::DjotContent.new(djot_hash)

      expect(djot.metadata).to be_a(Hash)
    end
  end

  describe 'DjotContent::FormattedBlock' do
    it 'initializes from hash' do
      block_hash = {
        block_type: 'paragraph',
        level: nil,
        content: 'block content',
        children: nil,
        attributes: {}
      }
      block = Kreuzberg::Result::DjotContent::FormattedBlock.new(block_hash)

      expect(block).to be_a(Kreuzberg::Result::DjotContent::FormattedBlock)
      expect(block.content).to eq('block content')
    end

    it 'to_h produces hash' do
      block_hash = {
        block_type: 'heading',
        level: 2,
        content: 'heading text',
        children: nil,
        attributes: {}
      }
      block = Kreuzberg::Result::DjotContent::FormattedBlock.new(block_hash)
      hash = block.to_h

      expect(hash).to be_a(Hash)
      expect(hash[:block_type]).to eq('heading')
      expect(hash[:level]).to eq(2)
    end

    it 'has block_type attribute' do
      block_hash = {
        block_type: 'list',
        level: nil,
        content: nil,
        children: [],
        attributes: {}
      }
      block = Kreuzberg::Result::DjotContent::FormattedBlock.new(block_hash)

      expect(block.block_type).to eq('list')
    end

    it 'handles children array' do
      child_hash = {
        block_type: 'paragraph',
        level: nil,
        content: 'child',
        children: nil,
        attributes: {}
      }
      block_hash = {
        block_type: 'container',
        level: nil,
        content: nil,
        children: [child_hash],
        attributes: {}
      }
      block = Kreuzberg::Result::DjotContent::FormattedBlock.new(block_hash)

      expect(block.children).to be_an(Array)
    end
  end

  describe 'DjotContent::DjotImage' do
    it 'initializes from hash' do
      image_hash = {
        url: 'https://example.com/image.png',
        alt: 'alt text',
        title: 'image title',
        attributes: {}
      }
      image = Kreuzberg::Result::DjotContent::DjotImage.new(image_hash)

      expect(image).to be_a(Kreuzberg::Result::DjotContent::DjotImage)
      expect(image.url).to eq('https://example.com/image.png')
    end

    it 'to_h produces hash' do
      image_hash = {
        url: 'image.png',
        alt: 'description',
        title: 'title',
        attributes: {}
      }
      image = Kreuzberg::Result::DjotContent::DjotImage.new(image_hash)
      hash = image.to_h

      expect(hash).to be_a(Hash)
      expect(hash[:url]).to eq('image.png')
      expect(hash[:alt]).to eq('description')
    end

    it 'handles missing optional fields' do
      image_hash = {
        url: 'image.png',
        alt: nil,
        title: nil,
        attributes: {}
      }
      image = Kreuzberg::Result::DjotContent::DjotImage.new(image_hash)

      expect(image.url).to eq('image.png')
      expect(image.alt).to be_nil
    end
  end

  describe 'DjotContent::DjotLink' do
    it 'initializes from hash' do
      link_hash = {
        url: 'https://example.com',
        text: 'link text',
        title: 'link title',
        link_type: 'external'
      }
      link = Kreuzberg::Result::DjotContent::DjotLink.new(link_hash)

      expect(link).to be_a(Kreuzberg::Result::DjotContent::DjotLink)
      expect(link.url).to eq('https://example.com')
      expect(link.text).to eq('link text')
    end

    it 'to_h produces hash' do
      link_hash = {
        url: '/page',
        text: 'internal link',
        title: 'page',
        link_type: 'internal'
      }
      link = Kreuzberg::Result::DjotContent::DjotLink.new(link_hash)
      hash = link.to_h

      expect(hash).to be_a(Hash)
      expect(hash[:url]).to eq('/page')
      expect(hash[:text]).to eq('internal link')
    end
  end

  describe 'DjotContent::Footnote' do
    it 'initializes with label and content' do
      footnote = Kreuzberg::Result::DjotContent::Footnote.new(
        label: 'fn1',
        content: 'Footnote content here'
      )

      expect(footnote).to be_a(Kreuzberg::Result::DjotContent::Footnote)
      expect(footnote.label).to eq('fn1')
      expect(footnote.content).to eq('Footnote content here')
    end

    it 'to_h produces hash' do
      footnote = Kreuzberg::Result::DjotContent::Footnote.new(
        label: 'note2',
        content: 'Some reference'
      )
      hash = footnote.to_h

      expect(hash).to be_a(Hash)
      expect(hash[:label]).to eq('note2')
      expect(hash[:content]).to eq('Some reference')
    end
  end

  describe 'Result with DjotContent' do
    it 'Result can contain DjotContent' do
      djot_hash = {
        plain_text: 'djot content',
        blocks: [],
        metadata_json: '{}',
        tables: [],
        images: [],
        links: [],
        footnotes: []
      }
      result = Kreuzberg::Result.new(
        content: 'original',
        mime_type: 'text/djot',
        metadata_json: '{}',
        tables: [],
        chunks: [],
        images: [],
        djot_content: djot_hash
      )

      expect(result.djot_content).to be_a(Kreuzberg::Result::DjotContent)
    end
  end
end
