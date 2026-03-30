<?php

declare(strict_types=1);

namespace Kreuzberg\Types\Metadata;

/**
 * JATS (Journal Article Tag Suite) metadata.
 *
 * Contains copyright, license, publication history dates,
 * and contributor role information.
 */
readonly class JatsMetadata
{
    /**
     * @param array<string, string> $historyDates
     * @param ContributorRole[] $contributorRoles
     */
    public function __construct(
        public ?string $copyright = null,
        public ?string $license = null,
        public array $historyDates = [],
        public array $contributorRoles = [],
    ) {
    }

    /**
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        /** @var string|null $copyright */
        $copyright = $data['copyright'] ?? null;

        /** @var string|null $license */
        $license = $data['license'] ?? null;

        /** @var array<string, string> $historyDates */
        $historyDates = $data['history_dates'] ?? [];
        if (!is_array($historyDates)) {
            $historyDates = [];
        }

        /** @var ContributorRole[] $contributorRoles */
        $contributorRoles = [];
        if (isset($data['contributor_roles']) && is_array($data['contributor_roles'])) {
            foreach ($data['contributor_roles'] as $roleData) {
                if (is_array($roleData)) {
                    $contributorRoles[] = ContributorRole::fromArray($roleData);
                }
            }
        }

        return new self(
            copyright: $copyright,
            license: $license,
            historyDates: $historyDates,
            contributorRoles: $contributorRoles,
        );
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        $result = [];

        if ($this->copyright !== null) {
            $result['copyright'] = $this->copyright;
        }

        if ($this->license !== null) {
            $result['license'] = $this->license;
        }

        if ($this->historyDates !== []) {
            $result['history_dates'] = $this->historyDates;
        }

        if ($this->contributorRoles !== []) {
            $result['contributor_roles'] = array_map(
                fn(ContributorRole $c) => $c->toArray(),
                $this->contributorRoles,
            );
        }

        return $result;
    }
}
