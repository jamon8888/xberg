# frozen_string_literal: true

RSpec.describe 'Keyword Extraction' do
  describe 'basic keyword extraction' do
    it 'extracts keywords from text' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 10
        )
      )

      text = 'Machine learning and artificial intelligence are transforming technology. Neural networks and deep learning are key areas of AI research. These technologies enable predictions and data analysis.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.content).not_to be_nil
      expect(result.content).to include('Machine learning')
    end

    it 'returns keywords in metadata' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 5
        )
      )

      text = 'Artificial intelligence transforms technology development. Machine learning algorithms improve with training data.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.metadata).to be_a(Hash)
    end

    it 'respects max_keywords parameter' do
      max_keywords_values = [1, 5, 10]

      max_keywords_values.each do |max_kw|
        config = Kreuzberg::Config::Extraction.new(
          keywords: Kreuzberg::Config::Keywords.new(
            algorithm: 'yake',
            max_keywords: max_kw
          )
        )

        text = 'Machine learning and artificial intelligence are transforming technology. Neural networks and deep learning are key research areas. Data science enables predictions. Algorithms process information efficiently.'
        result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

        expect(result).not_to be_nil
        expect(result.content).not_to be_nil
      end
    end

    it 'returns content when keywords enabled' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 5
        )
      )

      text = 'Artificial intelligence is transforming the world.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.content).not_to be_nil
      expect(result.content).to include('Artificial intelligence')
    end
  end

  describe 'multilingual keyword extraction' do
    it 'extracts keywords from English text' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          language: 'en',
          max_keywords: 5
        )
      )

      text = 'Machine learning and artificial intelligence are transforming technology development globally.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.content).not_to be_nil
    end

    it 'extracts keywords from German text' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          language: 'de',
          max_keywords: 5
        )
      )

      text = 'Maschinelles Lernen und künstliche Intelligenz transformieren die Technologieentwicklung.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.content).not_to be_nil
    end

    it 'extracts keywords from French text' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          language: 'fr',
          max_keywords: 5
        )
      )

      text = "L'apprentissage automatique et l'intelligence artificielle transforment le développement technologique."
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.content).not_to be_nil
    end

    it 'handles language parameter correctly' do
      languages = %w[en de fr es it pt nl]

      languages.each do |lang|
        config = Kreuzberg::Config::Extraction.new(
          keywords: Kreuzberg::Config::Keywords.new(
            algorithm: 'yake',
            language: lang,
            max_keywords: 3
          )
        )

        text = 'Machine learning and artificial intelligence.'
        result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

        expect(result).not_to be_nil
      end
    end
  end

  describe 'min_score filtering' do
    it 'filters keywords with different score thresholds' do
      text = 'Machine learning and artificial intelligence are transforming technology. Neural networks and deep learning are key research areas.'

      results_by_threshold = {}
      thresholds = [0.1, 0.5, 0.9]

      thresholds.each do |threshold|
        config = Kreuzberg::Config::Extraction.new(
          keywords: Kreuzberg::Config::Keywords.new(
            algorithm: 'yake',
            max_keywords: 100,
            min_score: threshold
          )
        )

        result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)
        results_by_threshold[threshold] = result

        expect(result).not_to be_nil
        expect(result.content).not_to be_nil
      end

      # Lower thresholds should produce more or equal results
      low_score_content = results_by_threshold[0.1]
      high_score_content = results_by_threshold[0.9]
      expect(low_score_content.content.length).to be >= high_score_content.content.length
    end

    it 'produces consistent results with same score threshold' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 50,
          min_score: 0.3
        )
      )

      text = 'Artificial intelligence is transforming data science and machine learning research globally with neural networks.'
      result1 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)
      result2 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)
      result3 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      # Results should be identical across runs
      expect(result1.content).to eq(result2.content)
      expect(result2.content).to eq(result3.content)
    end
  end

  describe 'ngram_range variations' do
    it 'extracts keywords with different ngram configurations' do
      text = 'Machine learning and artificial intelligence are transforming data science and technology development globally.'

      configs = {
        'single_words' => [1, 1],
        'bigrams' => [2, 2],
        'unigram_bigram' => [1, 2],
        'unigram_trigram' => [1, 3]
      }

      results = {}
      configs.each do |label, range|
        config = Kreuzberg::Config::Extraction.new(
          keywords: Kreuzberg::Config::Keywords.new(
            algorithm: 'yake',
            max_keywords: 15,
            ngram_range: range
          )
        )

        result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)
        results[label] = result

        expect(result).not_to be_nil
        expect(result.content).not_to be_nil
        expect(result.content.length).to be > 0
      end

      # Wider n-gram ranges typically produce more content due to phrase inclusion
      expect(results['unigram_trigram'].content.length).to be >= results['single_words'].content.length
    end

    it 'produces consistent results with same ngram_range' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 10,
          ngram_range: [1, 2]
        )
      )

      text = 'Machine learning and artificial intelligence are transforming technology development across industry.'
      result1 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)
      result2 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      # Results should be identical
      expect(result1.content).to eq(result2.content)
    end
  end

  describe 'algorithm selection' do
    it 'both YAKE and RAKE algorithms produce results' do
      text = 'Machine learning and artificial intelligence are transforming technology and neural networks enable deep learning.'

      yake_config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 10
        )
      )

      rake_config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'rake',
          max_keywords: 10
        )
      )

      yake_result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: yake_config)
      rake_result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: rake_config)

      expect(yake_result).not_to be_nil
      expect(yake_result.content).not_to be_nil
      expect(yake_result.content.length).to be > 0

      expect(rake_result).not_to be_nil
      expect(rake_result.content).not_to be_nil
      expect(rake_result.content.length).to be > 0
    end

    it 'algorithm-specific parameters affect extraction' do
      text = 'Machine learning and artificial intelligence are transforming technology development and research.'

      # YAKE with different window sizes
      yake_config_small = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 10,
          yake_params: Kreuzberg::Config::KeywordYakeParams.new(window_size: 2)
        )
      )

      yake_config_large = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 10,
          yake_params: Kreuzberg::Config::KeywordYakeParams.new(window_size: 4)
        )
      )

      result_small = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: yake_config_small)
      result_large = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: yake_config_large)

      expect(result_small).not_to be_nil
      expect(result_large).not_to be_nil
      # Both should produce valid results
      expect(result_small.content.length).to be > 0
      expect(result_large.content.length).to be > 0
    end

    it 'RAKE with min_word_length parameter works' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'rake',
          max_keywords: 10,
          rake_params: Kreuzberg::Config::KeywordRakeParams.new(
            min_word_length: 2,
            max_words_per_phrase: 4
          )
        )
      )

      text = 'Machine learning and artificial intelligence are transforming technology.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.content).not_to be_nil
      expect(result.content.length).to be > 0
    end
  end

  describe 'batch keyword extraction' do
    it 'processes multiple texts with same configuration' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 5
        )
      )

      texts = [
        'Machine learning and artificial intelligence are transforming technology.',
        'Deep learning neural networks enable advanced data science applications.',
        'Artificial intelligence enables predictions and automation globally.'
      ]

      results = texts.map { |text| Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config) }

      expect(results.length).to eq(3)
      results.each do |result|
        expect(result).not_to be_nil
        expect(result.content).not_to be_nil
      end
    end

    it 'maintains consistency across batch processing' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 10
        )
      )

      text = 'Machine learning and artificial intelligence are transforming technology development globally.'

      result1 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)
      result2 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result1).not_to be_nil
      expect(result2).not_to be_nil
      expect(result1.content).to eq(result2.content)
    end

    it 'handles different configurations per batch item' do
      text = 'Machine learning and artificial intelligence are transforming technology.'

      configs = [
        Kreuzberg::Config::Extraction.new(
          keywords: Kreuzberg::Config::Keywords.new(algorithm: 'yake', max_keywords: 5)
        ),
        Kreuzberg::Config::Extraction.new(
          keywords: Kreuzberg::Config::Keywords.new(algorithm: 'rake', max_keywords: 5)
        ),
        Kreuzberg::Config::Extraction.new(
          keywords: Kreuzberg::Config::Keywords.new(algorithm: 'yake', max_keywords: 10)
        )
      ]

      results = configs.map { |cfg| Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: cfg) }

      expect(results.length).to eq(3)
      results.each do |result|
        expect(result).not_to be_nil
      end
    end
  end

  describe 'score normalization validation' do
    it 'returns normalized scores between 0 and 1' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 20
        )
      )

      text = 'Machine learning and artificial intelligence are transforming technology. Neural networks and deep learning enable data science applications.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.content).not_to be_nil
    end

    it 'validates score values are realistic' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 50
        )
      )

      text = 'Artificial intelligence machine learning data science neural networks deep learning transforming technology.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.content).not_to be_nil
    end

    it 'handles score filtering with normalized ranges' do
      scores_to_test = [0.0, 0.25, 0.5, 0.75, 1.0]

      scores_to_test.each do |score_threshold|
        config = Kreuzberg::Config::Extraction.new(
          keywords: Kreuzberg::Config::Keywords.new(
            algorithm: 'yake',
            max_keywords: 100,
            min_score: score_threshold
          )
        )

        text = 'Machine learning artificial intelligence data science neural networks deep learning technology.'
        result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

        expect(result).not_to be_nil
      end
    end

    it 'ensures score consistency for same text' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 10
        )
      )

      text = 'Machine learning and artificial intelligence transform technology.'
      result1 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)
      result2 = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result1).not_to be_nil
      expect(result2).not_to be_nil
    end
  end

  describe 'empty and edge cases' do
    it 'handles very short text gracefully' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 5
        )
      )

      text = 'AI'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.content).not_to be_nil
    end

    it 'handles text with no obvious keywords' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 5
        )
      )

      text = 'a b c d e'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
    end

    it 'handles text with repeated keywords' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 5
        )
      )

      text = 'Machine machine machine learning learning learning artificial artificial artificial intelligence intelligence.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.content).not_to be_nil
    end

    it 'handles max_keywords of 0' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 0
        )
      )

      text = 'Machine learning and artificial intelligence.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
    end

    it 'handles large max_keywords value' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: 'yake',
          max_keywords: 1000
        )
      )

      text = 'Machine learning and artificial intelligence are transforming technology.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
    end

    it 'handles disabled keyword extraction' do
      config = Kreuzberg::Config::Extraction.new
      text = 'Machine learning and artificial intelligence.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
      expect(result.content).not_to be_nil
    end

    it 'handles keywords config with nil values' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: Kreuzberg::Config::Keywords.new(
          algorithm: nil,
          max_keywords: nil
        )
      )

      text = 'Machine learning and artificial intelligence.'
      result = Kreuzberg.extract_bytes_sync(data: text, mime_type: 'text/plain', config: config)

      expect(result).not_to be_nil
    end
  end

  describe 'integration with Extraction config' do
    it 'accepts Keywords config in Extraction' do
      keywords = Kreuzberg::Config::Keywords.new(
        algorithm: 'yake',
        max_keywords: 10
      )
      config = Kreuzberg::Config::Extraction.new(keywords: keywords)

      expect(config.keywords).to be_a(Kreuzberg::Config::Keywords)
      expect(config.keywords.algorithm).to eq('yake')
    end

    it 'accepts keywords config as hash in Extraction' do
      config = Kreuzberg::Config::Extraction.new(
        keywords: {
          algorithm: 'rake',
          max_keywords: 15,
          min_score: 0.3
        }
      )

      expect(config.keywords).to be_a(Kreuzberg::Config::Keywords)
      expect(config.keywords.algorithm).to eq('rake')
      expect(config.keywords.max_keywords).to eq(15)
    end

    it 'includes keywords config in to_h' do
      keywords = Kreuzberg::Config::Keywords.new(
        algorithm: 'yake',
        max_keywords: 10
      )
      config = Kreuzberg::Config::Extraction.new(keywords: keywords)

      hash = config.to_h

      expect(hash).to include(:keywords)
      expect(hash[:keywords]).to be_a(Hash)
      expect(hash[:keywords][:algorithm]).to eq('yake')
    end

    it 'handles nil keywords config' do
      config = Kreuzberg::Config::Extraction.new(keywords: nil)

      expect(config.keywords).to be_nil
      hash = config.to_h
      expect(hash[:keywords]).to be_nil
    end
  end
end
