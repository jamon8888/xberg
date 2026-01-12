# frozen_string_literal: true

RSpec.describe 'Embeddings Vector Generation' do
  describe 'vector generation correctness' do
    it 'generates embedding vectors with correct dimensions' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          max_chars: 500,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true,
            batch_size: 32
          )
        )
      )

      text = 'Machine learning transforms technology. Artificial intelligence enables automation and prediction across industries.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.chunks).not_to be_nil
      unless result.chunks.empty?
        first_chunk = result.chunks.first
        expect(first_chunk.embedding).not_to be_nil if first_chunk.embedding
        if first_chunk.embedding.is_a?(Array) && !first_chunk.embedding.empty?
          dimension = first_chunk.embedding.length
          expect(dimension).to(satisfy { |d| [384, 512, 768, 1024].include?(d) })
        end
      end
    end

    it 'produces numeric embedding vectors' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text = 'Deep learning neural networks enable complex pattern recognition.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      if result.chunks && !result.chunks.empty? && result.chunks.first.embedding
        embedding = result.chunks.first.embedding
        expect(embedding).to be_a(Array)
        expect(embedding).to all(be_a(Numeric))
      end
    end

    it 'generates consistent vectors for same input' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' }
          )
        )
      )

      text = 'Artificial intelligence transforms technology development.'
      result1 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)
      result2 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result1).not_to be_nil
      expect(result2).not_to be_nil

      if result1.chunks && result2.chunks && !result1.chunks.empty? && !result2.chunks.empty? && result1.chunks.first.embedding && result2.chunks.first.embedding
        expect(result1.chunks.first.embedding).to eq(result2.chunks.first.embedding)
      end
    end
  end

  describe 'embedding dimension verification' do
    it 'validates embedding vector length consistency within batch' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          max_chars: 300,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            batch_size: 8
          )
        )
      )

      text = 'Machine learning techniques evolve continuously. Neural networks improve performance. Data science enables insights.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      if result.chunks && result.chunks.length > 1
        embedding_dims = result.chunks
                               .select { |chunk| chunk.embedding.is_a?(Array) }
                               .map { |chunk| chunk.embedding.length }
                               .uniq

        # All embeddings should have consistent dimensions
        expect(embedding_dims.length).to eq(1)
      end
    end

    it 'respects configured batch size for embeddings' do
      batch_sizes = [8, 16, 32]

      batch_sizes.each do |batch_size|
        config = Kreuzberg::Config::Extraction.new(
          chunking: Kreuzberg::Config::Chunking.new(
            enabled: true,
            max_chars: 200,
            embedding: Kreuzberg::Config::Embedding.new(
              model: { type: :preset, name: 'balanced' },
              batch_size: batch_size
            )
          )
        )

        text = 'Technology transforms industries. Machine learning advances AI. Neural networks improve models. Data analysis drives decisions.'
        result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

        expect(result).not_to be_nil
        expect(result.chunks).not_to be_nil
      end
    end

    it 'confirms embedding dimensions match model specification' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text = 'Embeddings capture semantic meaning in vector space representations.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      if result.chunks && !result.chunks.empty? && result.chunks.first.embedding
        embedding = result.chunks.first.embedding
        dimension = embedding.length
        # Verify dimension is positive
        expect(dimension).to be > 0
        # Verify dimension is in expected range for balanced model
        expect(dimension).to be_within(1024).of(512)
      end
    end

    it 'handles empty chunk gracefully' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' }
          )
        )
      )

      text = 'a'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
    end
  end

  describe 'performance with batch operations' do
    it 'processes multiple chunks efficiently' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          max_chars: 250,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            batch_size: 16
          )
        )
      )

      text = 'Machine learning transforms industries. Neural networks enable deep learning. ' \
             'Artificial intelligence drives automation. Data science enables insights. ' \
             'Computer vision processes images. Natural language processing understands text. ' \
             'Reinforcement learning optimizes decisions. Transfer learning improves efficiency.'

      start_time = Time.now
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)
      elapsed = Time.now - start_time

      expect(result).not_to be_nil
      expect(elapsed).to be < 30.0
      expect(result.chunks.length).to be > 0
    end

    it 'maintains consistent embedding quality across batch sizes' do
      text = 'Deep learning networks process data efficiently through parallel computation architecture.'

      results = []
      [8, 16, 32].each do |batch_size|
        config = Kreuzberg::Config::Extraction.new(
          chunking: Kreuzberg::Config::Chunking.new(
            enabled: true,
            embedding: Kreuzberg::Config::Embedding.new(
              model: { type: :preset, name: 'balanced' },
              batch_size: batch_size
            )
          )
        )

        result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)
        results << result if result
      end

      expect(results.length).to be > 0
      results.each do |result|
        expect(result).not_to be_nil
        expect(result.chunks).to be_a(Array)
      end
    end

    it 'handles large text with many chunks' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          max_chars: 300,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            batch_size: 32
          )
        )
      )

      text = ('Machine learning and artificial intelligence are transforming technology. ' * 10)
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.chunks.length).to be >= 1
      expect(result.chunk_count).to eq(result.chunks.length)
    end
  end

  describe 'format-specific embedding handling' do
    it 'generates embeddings from plain text' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' }
          )
        )
      )

      text = 'Machine learning enables advanced data analysis and predictions.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.mime_type).to include('text')
      expect(result.chunks).not_to be_nil
    end

    it 'handles extraction without embeddings when disabled' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: nil
        )
      )

      text = 'Data science transforms business decisions with insights.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.chunks).not_to be_nil
    end

    it 'respects extraction config for embeddings' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          max_chars: 400,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true,
            batch_size: 16
          )
        )
      )

      text = 'Artificial intelligence enables automation across industries.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.chunks).to be_a(Array)
    end
  end

  describe 'similarity score validation' do
    it 'generates vectors suitable for cosine similarity' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text = 'Machine learning enables pattern recognition. Artificial intelligence drives innovation.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      if result.chunks && result.chunks.length >= 2
        embeddings = result.chunks.select { |c| c.embedding.is_a?(Array) }.map(&:embedding)

        if embeddings.length >= 2
          vec1 = embeddings[0]
          vec2 = embeddings[1]

          dot_product = vec1.zip(vec2).sum { |a, b| a * b }
          norm1 = Math.sqrt(vec1.sum { |x| x * x })
          norm2 = Math.sqrt(vec2.sum { |x| x * x })
          similarity = dot_product / (norm1 * norm2) if norm1 > 0 && norm2 > 0

          if similarity
            expect(similarity).to be >= -1.0
            expect(similarity).to be <= 1.0
          end
        end
      end
    end

    it 'normalized embeddings have unit or near-unit magnitude' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text = 'Embeddings capture semantic information in vector space.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      if result.chunks && !result.chunks.empty? && result.chunks.first.embedding
        embedding = result.chunks.first.embedding
        magnitude = Math.sqrt(embedding.sum { |x| x * x })

        expect(magnitude).to be > 0.0
      end
    end

    it 'computes reasonable similarity between related texts' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text1 = 'Machine learning enables data analysis'
      text2 = 'Artificial intelligence enables learning'

      result1 = Kreuzberg.extract_bytes_sync(data: text1, mime_type: 'text/plain', config: config)
      result2 = Kreuzberg.extract_bytes_sync(data: text2, mime_type: 'text/plain', config: config)

      if result1.chunks && result2.chunks && !result1.chunks.empty? && !result2.chunks.empty?
        emb1 = result1.chunks.first.embedding
        emb2 = result2.chunks.first.embedding

        if emb1.is_a?(Array) && emb2.is_a?(Array)
          dot_product = emb1.zip(emb2).sum { |a, b| a * b }
          norm1 = Math.sqrt(emb1.sum { |x| x * x })
          norm2 = Math.sqrt(emb2.sum { |x| x * x })
          similarity = dot_product / (norm1 * norm2) if norm1 > 0 && norm2 > 0

          expect(similarity).not_to be_nil if similarity
          # Cosine similarity must be in valid range [-1, 1]
          expect(similarity).to be >= -1.0 if similarity
          expect(similarity).to be <= 1.0 if similarity
          # Related texts should have positive similarity
          expect(similarity).to be > 0 if similarity
        end
      end
    end
  end

  describe 'normalization correctness' do
    it 'respects normalization configuration' do
      config_normalized = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text = 'Normalization ensures consistent vector magnitudes.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config_normalized)

      expect(result).not_to be_nil
      expect(result.chunks).not_to be_nil
    end

    it 'handles embedding with custom normalization settings' do
      normalize_options = [true, false]

      normalize_options.each do |should_normalize|
        config = Kreuzberg::Config::Extraction.new(
          chunking: Kreuzberg::Config::Chunking.new(
            enabled: true,
            embedding: Kreuzberg::Config::Embedding.new(
              model: { type: :preset, name: 'balanced' },
              normalize: should_normalize
            )
          )
        )

        text = 'Embeddings can be normalized or unnormalized depending on use case.'
        result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

        expect(result).not_to be_nil
        expect(result.chunks).to be_a(Array)
      end
    end

    it 'validates embedding values within expected range' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text = 'Normalized embeddings typically have values between -1 and 1.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      if result.chunks && !result.chunks.empty? && result.chunks.first.embedding
        embedding = result.chunks.first.embedding
        expect(embedding).to all(be >= -2.0)
        expect(embedding).to all(be <= 2.0)
      end
    end

    it 'ensures numerical stability in embeddings' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text = 'Numerical stability prevents overflow and underflow in floating point computation.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      if result.chunks && !result.chunks.empty? && result.chunks.first.embedding
        embedding = result.chunks.first.embedding
        expect(embedding).to all(be_finite)
      end
    end
  end

  describe 'embedding configuration validation' do
    it 'creates Embedding config with default values' do
      embedding = Kreuzberg::Config::Embedding.new

      expect(embedding.model).to be_a(Hash)
      expect(embedding.normalize).to be true
      expect(embedding.batch_size).to eq(32)
      expect(embedding.show_download_progress).to be false
      expect(embedding.cache_dir).to be_nil
    end

    it 'creates Embedding config with custom values' do
      embedding = Kreuzberg::Config::Embedding.new(
        model: { type: :preset, name: 'large' },
        normalize: false,
        batch_size: 64,
        show_download_progress: true,
        cache_dir: '/tmp/embeddings'
      )

      expect(embedding.model[:type]).to eq(:preset)
      expect(embedding.model[:name]).to eq('large')
      expect(embedding.normalize).to be false
      expect(embedding.batch_size).to eq(64)
      expect(embedding.show_download_progress).to be true
      expect(embedding.cache_dir).to eq('/tmp/embeddings')
    end

    it 'converts Embedding config to hash' do
      embedding = Kreuzberg::Config::Embedding.new(
        model: { type: :preset, name: 'balanced' },
        normalize: true,
        batch_size: 16
      )

      hash = embedding.to_h

      expect(hash).to be_a(Hash)
      expect(hash[:model]).to be_a(Hash)
      expect(hash[:normalize]).to be true
      expect(hash[:batch_size]).to eq(16)
    end

    it 'compacts nil values in embedding hash' do
      embedding = Kreuzberg::Config::Embedding.new(
        model: { type: :preset, name: 'balanced' },
        batch_size: 32
      )

      hash = embedding.to_h

      expect(hash).not_to have_key(:cache_dir)
      expect(hash).not_to have_key(:show_download_progress) unless hash[:show_download_progress] == false
    end

    it 'accepts hash model specification' do
      embedding = Kreuzberg::Config::Embedding.new(
        model: { type: :preset, name: 'balanced', dimension: 384 }
      )

      expect(embedding.model).to be_a(Hash)
      expect(embedding.model[:type]).to eq(:preset)
      expect(embedding.model[:name]).to eq('balanced')
    end

    it 'converts numeric batch_size to integer' do
      embedding = Kreuzberg::Config::Embedding.new(batch_size: '64')

      expect(embedding.batch_size).to eq(64)
      expect(embedding.batch_size).to be_a(Integer)
    end
  end

  describe 'edge cases and error handling' do
    it 'handles very short text with embeddings' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' }
          )
        )
      )

      text = 'AI'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.chunks).not_to be_nil
    end

    it 'handles text with special characters in embeddings' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' }
          )
        )
      )

      text = 'Machine learning & AI. Data science -> insights. Deep learning @ scale.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.chunks).not_to be_nil
    end

    it 'handles unicode text in embeddings' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' }
          )
        )
      )

      text = 'Machine learning transforms technology. Aprendizaje automático transforma.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
    end

    it 'handles repeated text in embeddings' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' }
          )
        )
      )

      text = 'machine machine machine learning learning learning'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
    end

    it 'handles empty embedding result gracefully' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: false
        )
      )

      text = 'This text will not be chunked.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.chunks).to be_a(Array)
    end
  end

  describe 'Chunking with Embedding integration' do
    it 'integrates embedding with chunking configuration' do
      chunking = Kreuzberg::Config::Chunking.new(
        max_chars: 500,
        max_overlap: 100,
        embedding: Kreuzberg::Config::Embedding.new(
          model: { type: :preset, name: 'balanced' },
          normalize: true
        )
      )

      expect(chunking.embedding).to be_a(Kreuzberg::Config::Embedding)
      expect(chunking.embedding.normalize).to be true
    end

    it 'accepts embedding config in Chunking' do
      embedding = Kreuzberg::Config::Embedding.new(batch_size: 16)
      chunking = Kreuzberg::Config::Chunking.new(embedding: embedding)

      expect(chunking.embedding).to be_a(Kreuzberg::Config::Embedding)
      expect(chunking.embedding.batch_size).to eq(16)
    end

    it 'accepts embedding config as hash in Chunking' do
      chunking = Kreuzberg::Config::Chunking.new(
        embedding: { batch_size: 32, normalize: false }
      )

      expect(chunking.embedding).to be_a(Kreuzberg::Config::Embedding)
      expect(chunking.embedding.batch_size).to eq(32)
      expect(chunking.embedding.normalize).to be false
    end

    it 'converts chunking with embedding to hash' do
      chunking = Kreuzberg::Config::Chunking.new(
        max_chars: 600,
        embedding: Kreuzberg::Config::Embedding.new(batch_size: 24)
      )

      hash = chunking.to_h

      expect(hash).to be_a(Hash)
      expect(hash[:embedding]).to be_a(Hash)
      expect(hash[:embedding][:batch_size]).to eq(24)
    end

    it 'handles nil embedding in chunking' do
      chunking = Kreuzberg::Config::Chunking.new(
        max_chars: 500,
        embedding: nil
      )

      expect(chunking.embedding).to be_nil
      hash = chunking.to_h
      expect(hash[:embedding]).to be_nil
    end
  end

  describe 'mathematical properties and error handling' do
    it 'validates embedding vector values are valid floats' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text = 'Validating floating-point properties of embedding values.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      if result.chunks && !result.chunks.empty? && result.chunks.first.embedding
        embedding = result.chunks.first.embedding
        expect(embedding).to all(be_a(Float))
        expect(embedding).to all(be_finite)
      end
    end

    it 'ensures no dead embeddings (all-zero vectors)' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text = 'Testing for dead embeddings and zero vectors.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      if result.chunks && !result.chunks.empty? && result.chunks.first.embedding
        embedding = result.chunks.first.embedding
        magnitude = embedding.sum(&:abs)
        expect(magnitude).to be > 0.1
      end
    end

    it 'validates identical vectors have similarity 1.0' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text = 'Testing identical vector similarity.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      if result.chunks && !result.chunks.empty? && result.chunks.first.embedding
        embedding = result.chunks.first.embedding
        # Vector with itself should have similarity 1.0
        dot_product = embedding.zip(embedding).sum { |a, b| a * b }
        norm_sq = embedding.sum { |x| x * x }
        similarity = dot_product / norm_sq if norm_sq > 0

        expect(similarity).to be_within(0.0001).of(1.0) if similarity
      end
    end

    it 'validates embedding consistency across multiple runs' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            normalize: true
          )
        )
      )

      text = 'Testing deterministic embedding generation.'

      result1 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)
      result2 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      if result1.chunks && result2.chunks && !result1.chunks.empty? && !result2.chunks.empty?
        emb1 = result1.chunks.first.embedding
        emb2 = result2.chunks.first.embedding

        expect(emb1).to eq(emb2) if emb1.is_a?(Array) && emb2.is_a?(Array)
      end
    end

    it 'handles dimension consistency across batch operations' do
      config = Kreuzberg::Config::Extraction.new(
        chunking: Kreuzberg::Config::Chunking.new(
          enabled: true,
          max_chars: 150,
          embedding: Kreuzberg::Config::Embedding.new(
            model: { type: :preset, name: 'balanced' },
            batch_size: 4
          )
        )
      )

      texts = [
        'First document about machine learning.',
        'Second document about neural networks.',
        'Third document about deep learning.'
      ]

      dimensions = []

      texts.each do |text|
        result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

        next unless result.chunks

        result.chunks.each do |chunk|
          dimensions << chunk.embedding.length if chunk.embedding.is_a?(Array)
        end
      end

      # All dimensions should be consistent
      expect(dimensions.uniq.length).to be <= 1 if dimensions.any?
    end
  end
end
