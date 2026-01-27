# frozen_string_literal: true

# rubocop:disable RSpec/RepeatedExample
RSpec.describe 'Output Format and Result Format Configuration' do
  describe Kreuzberg::Config::Extraction do
    describe 'output_format' do
      it 'accepts output_format as initialization parameter' do
        config = described_class.new(output_format: 'markdown')

        expect(config.output_format).to eq 'markdown'
      end

      it 'defaults to nil when not specified' do
        config = described_class.new

        expect(config.output_format).to be_nil
      end

      it 'accepts plain format' do
        config = described_class.new(output_format: 'plain')

        expect(config.output_format).to eq 'plain'
      end

      it 'accepts markdown format' do
        config = described_class.new(output_format: 'markdown')

        expect(config.output_format).to eq 'markdown'
      end

      it 'accepts djot format' do
        config = described_class.new(output_format: 'djot')

        expect(config.output_format).to eq 'djot'
      end

      it 'accepts html format' do
        config = described_class.new(output_format: 'html')

        expect(config.output_format).to eq 'html'
      end

      it 'converts output_format to string' do
        config = described_class.new(output_format: :markdown)

        expect(config.output_format).to eq 'markdown'
        expect(config.output_format).to be_a String
      end

      it 'includes output_format in to_h' do
        config = described_class.new(output_format: 'markdown')
        hash = config.to_h

        expect(hash[:output_format]).to eq 'markdown'
      end

      it 'excludes nil output_format from to_h' do
        config = described_class.new(output_format: nil)
        hash = config.to_h

        expect(hash.key?(:output_format)).to be false
      end

      it 'includes output_format in JSON' do
        config = described_class.new(output_format: 'markdown')
        json = config.to_json
        parsed = JSON.parse(json)

        expect(parsed['output_format']).to eq 'markdown'
      end

      it 'retrieves output_format with get_field' do
        config = described_class.new(output_format: 'djot')

        expect(config.get_field('output_format')).to eq 'djot'
      end

      it 'can be set with []=' do
        config = described_class.new
        config[:output_format] = 'html'

        expect(config.output_format).to eq 'html'
      end

      it 'can be set with []= using symbol' do
        config = described_class.new
        config[:output_format] = :plain

        expect(config.output_format).to eq 'plain'
      end

      it 'can be retrieved with []' do
        config = described_class.new(output_format: 'markdown')

        expect(config[:output_format]).to eq 'markdown'
      end
    end

    describe 'result_format' do
      it 'accepts result_format as initialization parameter' do
        config = described_class.new(result_format: 'unified')

        expect(config.result_format).to eq 'unified'
      end

      it 'defaults to nil when not specified' do
        config = described_class.new

        expect(config.result_format).to be_nil
      end

      it 'accepts unified format' do
        config = described_class.new(result_format: 'unified')

        expect(config.result_format).to eq 'unified'
      end

      it 'accepts element_based format' do
        config = described_class.new(result_format: 'element_based')

        expect(config.result_format).to eq 'element_based'
      end

      it 'converts result_format to string' do
        config = described_class.new(result_format: :unified)

        expect(config.result_format).to eq 'unified'
        expect(config.result_format).to be_a String
      end

      it 'includes result_format in to_h' do
        config = described_class.new(result_format: 'element_based')
        hash = config.to_h

        expect(hash[:result_format]).to eq 'element_based'
      end

      it 'excludes nil result_format from to_h' do
        config = described_class.new(result_format: nil)
        hash = config.to_h

        expect(hash.key?(:result_format)).to be false
      end

      it 'includes result_format in JSON' do
        config = described_class.new(result_format: 'element_based')
        json = config.to_json
        parsed = JSON.parse(json)

        expect(parsed['result_format']).to eq 'element_based'
      end

      it 'retrieves result_format with get_field' do
        config = described_class.new(result_format: 'unified')

        expect(config.get_field('result_format')).to eq 'unified'
      end

      it 'can be set with []=' do
        config = described_class.new
        config[:result_format] = 'unified'

        expect(config.result_format).to eq 'unified'
      end

      it 'can be set with []= using symbol' do
        config = described_class.new
        config[:result_format] = :element_based

        expect(config.result_format).to eq 'element_based'
      end

      it 'can be retrieved with []' do
        config = described_class.new(result_format: 'element_based')

        expect(config[:result_format]).to eq 'element_based'
      end
    end

    describe 'combined output and result formats' do
      it 'accepts both output_format and result_format' do
        config = described_class.new(
          output_format: 'markdown',
          result_format: 'unified'
        )

        expect(config.output_format).to eq 'markdown'
        expect(config.result_format).to eq 'unified'
      end

      it 'serializes both formats in to_h' do
        config = described_class.new(
          output_format: 'djot',
          result_format: 'element_based'
        )
        hash = config.to_h

        expect(hash[:output_format]).to eq 'djot'
        expect(hash[:result_format]).to eq 'element_based'
      end

      it 'serializes both formats in JSON' do
        config = described_class.new(
          output_format: 'html',
          result_format: 'unified'
        )
        json = config.to_json
        parsed = JSON.parse(json)

        expect(parsed['output_format']).to eq 'html'
        expect(parsed['result_format']).to eq 'unified'
      end

      it 'merges both formats correctly' do
        base = described_class.new(
          output_format: 'markdown',
          result_format: 'unified'
        )
        override = described_class.new(output_format: 'html')
        merged = base.merge(override)

        expect(merged.output_format).to eq 'html'
        expect(merged.result_format).to eq 'unified'
      end

      it 'merges both formats with merge!' do
        config = described_class.new(
          output_format: 'markdown',
          result_format: 'unified'
        )
        override = described_class.new(
          output_format: 'djot',
          result_format: 'element_based'
        )
        config.merge!(override)

        expect(config.output_format).to eq 'djot'
        expect(config.result_format).to eq 'element_based'
      end

      it 'handles merge with hash containing both formats' do
        config = described_class.new(
          output_format: 'plain',
          result_format: 'unified'
        )
        merged = config.merge({ output_format: 'markdown' })

        expect(merged.output_format).to eq 'markdown'
        expect(merged.result_format).to eq 'unified'
      end
    end

    describe 'format persistence across operations' do
      it 'persists output_format through multiple conversions' do
        config = described_class.new(output_format: 'markdown')
        hash = config.to_h
        new_config = described_class.new(**hash)

        expect(new_config.output_format).to eq 'markdown'
      end

      it 'persists result_format through multiple conversions' do
        config = described_class.new(result_format: 'element_based')
        hash = config.to_h
        new_config = described_class.new(**hash)

        expect(new_config.result_format).to eq 'element_based'
      end

      it 'round-trips through JSON' do
        config = described_class.new(
          output_format: 'djot',
          result_format: 'unified'
        )
        json = config.to_json
        parsed = JSON.parse(json)
        new_config = described_class.new(**parsed.transform_keys(&:to_sym))

        expect(new_config.output_format).to eq 'djot'
        expect(new_config.result_format).to eq 'unified'
      end
    end

    describe 'format validation and edge cases' do
      it 'raises error for empty string output_format' do
        expect do
          described_class.new(output_format: '')
        end.to raise_error(ArgumentError, /Invalid output_format/)
      end

      it 'raises error for empty string result_format' do
        expect do
          described_class.new(result_format: '')
        end.to raise_error(ArgumentError, /Invalid result_format/)
      end

      it 'raises error for whitespace in output_format' do
        expect do
          described_class.new(output_format: '  plain  ')
        end.to raise_error(ArgumentError, /Invalid output_format/)
      end

      it 'normalizes case in output_format' do
        config = described_class.new(output_format: 'MarkDown')

        expect(config.output_format).to eq 'markdown'
      end

      it 'raises error for custom string in result_format' do
        expect do
          described_class.new(result_format: 'custom_format')
        end.to raise_error(ArgumentError, /Invalid result_format/)
      end
    end

    describe 'integration with other config fields' do
      it 'works with output_format and chunking together' do
        config = described_class.new(
          output_format: 'markdown',
          chunking: { max_chars: 500 }
        )

        expect(config.output_format).to eq 'markdown'
        expect(config.chunking.max_chars).to eq 500
      end

      it 'works with result_format and OCR together' do
        config = described_class.new(
          result_format: 'element_based',
          ocr: { backend: 'tesseract' }
        )

        expect(config.result_format).to eq 'element_based'
        expect(config.ocr.backend).to eq 'tesseract'
      end

      it 'works with both formats and language detection' do
        config = described_class.new(
          output_format: 'html',
          result_format: 'unified',
          language_detection: { enabled: true }
        )

        expect(config.output_format).to eq 'html'
        expect(config.result_format).to eq 'unified'
        expect(config.language_detection.enabled).to be true
      end

      it 'preserves formats in complex config merge' do
        base = described_class.new(
          output_format: 'markdown',
          result_format: 'unified',
          chunking: { max_chars: 500 },
          ocr: { backend: 'tesseract' }
        )
        override = described_class.new(
          output_format: 'djot',
          chunking: { max_chars: 750 }
        )
        merged = base.merge(override)

        expect(merged.output_format).to eq 'djot'
        expect(merged.result_format).to eq 'unified'
        expect(merged.chunking.max_chars).to eq 750
        expect(merged.ocr.backend).to eq 'tesseract'
      end
    end

    describe 'allowed keys integration' do
      it 'includes output_format in ALLOWED_KEYS' do
        expect(Kreuzberg::Config::Extraction::ALLOWED_KEYS).to include(:output_format)
      end

      it 'includes result_format in ALLOWED_KEYS' do
        expect(Kreuzberg::Config::Extraction::ALLOWED_KEYS).to include(:result_format)
      end
    end
  end
end
# rubocop:enable RSpec/RepeatedExample
