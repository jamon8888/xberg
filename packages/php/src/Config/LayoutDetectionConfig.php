<?php

declare(strict_types=1);

namespace Kreuzberg\Config;

/**
 * Layout detection configuration.
 *
 * Controls document layout analysis including region detection presets,
 * confidence filtering, heuristic post-processing, and table structure
 * recognition model selection.
 *
 * @example
 * ```php
 * use Kreuzberg\Config\LayoutDetectionConfig;
 *
 * $layout = new LayoutDetectionConfig(
 *     preset: 'accurate',
 *     confidenceThreshold: 0.75,
 *     applyHeuristics: true,
 *     tableModel: 'tatr',
 * );
 * ```
 */
readonly class LayoutDetectionConfig
{
    /**
     * @param string $preset Model preset controlling accuracy vs speed trade-off.
     *                       Supported values: "fast", "accurate". Default "accurate".
     * @param float|null $confidenceThreshold Minimum confidence threshold for detected layout
     *                                        regions (0.0-1.0). Regions below this threshold are discarded.
     *                                        Default null (use engine default).
     * @param bool $applyHeuristics Whether to apply heuristic post-processing to refine
     *                              layout regions. Default true.
     * @param string|null $tableModel Table structure recognition model to use.
     *                                Supported values: "tatr", "slanet_wired", "slanet_wireless", "slanet_plus", "slanet_auto".
     *                                Default null (use engine default).
     */
    public function __construct(
        public string $preset = 'accurate',
        public ?float $confidenceThreshold = null,
        public bool $applyHeuristics = true,
        public ?string $tableModel = null,
    ) {
    }

    /**
     * Create configuration from array data.
     *
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        $preset = $data['preset'] ?? 'accurate';
        $rawThreshold = $data['confidence_threshold'] ?? null;
        $confidenceThreshold = is_numeric($rawThreshold) ? (float) $rawThreshold : null;
        $applyHeuristics = isset($data['apply_heuristics']) ? (bool) $data['apply_heuristics'] : true;
        $rawTableModel = $data['table_model'] ?? null;
        $tableModel = is_string($rawTableModel) ? $rawTableModel : null;

        return new self(
            preset: is_string($preset) ? $preset : 'accurate',
            confidenceThreshold: $confidenceThreshold,
            applyHeuristics: $applyHeuristics,
            tableModel: $tableModel,
        );
    }

    /**
     * Convert configuration to array for FFI.
     *
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        $result = [
            'preset' => $this->preset,
            'apply_heuristics' => $this->applyHeuristics,
        ];

        if ($this->confidenceThreshold !== null) {
            $result['confidence_threshold'] = $this->confidenceThreshold;
        }

        if ($this->tableModel !== null) {
            $result['table_model'] = $this->tableModel;
        }

        return $result;
    }
}
