<?php

declare(strict_types=1);

namespace Kreuzberg\Types\Metadata;

/**
 * dBASE (DBF) file metadata.
 *
 * Contains record count, field count, and field definitions.
 */
readonly class DbfMetadata
{
    /**
     * @param DbfFieldInfo[] $fields
     */
    public function __construct(
        public int $recordCount = 0,
        public int $fieldCount = 0,
        public array $fields = [],
    ) {
    }

    /**
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        /** @var int $recordCount */
        $recordCount = (int) ($data['record_count'] ?? 0);

        /** @var int $fieldCount */
        $fieldCount = (int) ($data['field_count'] ?? 0);

        /** @var DbfFieldInfo[] $fields */
        $fields = [];
        if (isset($data['fields']) && is_array($data['fields'])) {
            foreach ($data['fields'] as $fieldData) {
                if (is_array($fieldData)) {
                    $fields[] = DbfFieldInfo::fromArray($fieldData);
                }
            }
        }

        return new self(
            recordCount: $recordCount,
            fieldCount: $fieldCount,
            fields: $fields,
        );
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        $result = [
            'record_count' => $this->recordCount,
            'field_count' => $this->fieldCount,
        ];

        if ($this->fields !== []) {
            $result['fields'] = array_map(
                fn(DbfFieldInfo $f) => $f->toArray(),
                $this->fields,
            );
        }

        return $result;
    }
}
