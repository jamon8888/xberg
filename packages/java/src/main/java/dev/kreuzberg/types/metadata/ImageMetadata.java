package dev.kreuzberg.types.metadata;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.Map;
import org.jspecify.annotations.Nullable;

/**
 * HTML image metadata.
 */
public record ImageMetadata(
    @JsonProperty("src") String src,
    @JsonProperty("alt") @Nullable String alt,
    @JsonProperty("title") @Nullable String title,
    @JsonProperty("dimensions") @Nullable int[] dimensions,
    @JsonProperty("image_type") String imageType,
    @JsonProperty("attributes") Map<String, String> attributes
) { }
