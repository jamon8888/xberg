<?php

declare(strict_types=1);

namespace Kreuzberg\Types\Metadata;

/**
 * dBASE field information.
 */
readonly class DbfFieldInfo
{
    public function __construct(
        public string $name,
        public string $fieldType,
    ) {
    }

    /**
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        return new self(
            name: (string) ($data['name'] ?? ''),
            fieldType: (string) ($data['field_type'] ?? ''),
        );
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        return [
            'name' => $this->name,
            'field_type' => $this->fieldType,
        ];
    }
}
