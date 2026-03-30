<?php

declare(strict_types=1);

namespace Kreuzberg\Types\Metadata;

/**
 * EPUB metadata (Dublin Core extensions).
 *
 * Contains extended Dublin Core metadata fields specific to EPUB files.
 */
readonly class EpubMetadata
{
    public function __construct(
        public ?string $coverage = null,
        public ?string $dcFormat = null,
        public ?string $relation = null,
        public ?string $source = null,
        public ?string $dcType = null,
        public ?string $coverImage = null,
    ) {
    }

    /**
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        return new self(
            coverage: $data['coverage'] ?? null,
            dcFormat: $data['dc_format'] ?? null,
            relation: $data['relation'] ?? null,
            source: $data['source'] ?? null,
            dcType: $data['dc_type'] ?? null,
            coverImage: $data['cover_image'] ?? null,
        );
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        $result = [];

        if ($this->coverage !== null) {
            $result['coverage'] = $this->coverage;
        }

        if ($this->dcFormat !== null) {
            $result['dc_format'] = $this->dcFormat;
        }

        if ($this->relation !== null) {
            $result['relation'] = $this->relation;
        }

        if ($this->source !== null) {
            $result['source'] = $this->source;
        }

        if ($this->dcType !== null) {
            $result['dc_type'] = $this->dcType;
        }

        if ($this->coverImage !== null) {
            $result['cover_image'] = $this->coverImage;
        }

        return $result;
    }
}
