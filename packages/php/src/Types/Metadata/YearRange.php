<?php

declare(strict_types=1);

namespace Kreuzberg\Types\Metadata;

/**
 * Year range for bibliographic metadata.
 */
readonly class YearRange
{
    /**
     * @param int[]$years
     */
    public function __construct(
        public ?int $min = null,
        public ?int $max = null,
        public array $years = [],
    ) {
    }

    /**
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        $min = isset($data['min']) ? (int) $data['min'] : null;
        $max = isset($data['max']) ? (int) $data['max'] : null;

        /** @var int[] $years */
        $years = $data['years'] ?? [];
        if (!is_array($years)) {
            $years = [];
        }

        return new self(
            min: $min,
            max: $max,
            years: array_map('intval', $years),
        );
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        $result = [];

        if ($this->min !== null) {
            $result['min'] = $this->min;
        }

        if ($this->max !== null) {
            $result['max'] = $this->max;
        }

        if ($this->years !== []) {
            $result['years'] = $this->years;
        }

        return $result;
    }
}
