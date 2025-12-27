package dev.kreuzberg.types.metadata;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;
import java.util.Map;
import org.jspecify.annotations.Nullable;

/**
 * HTML link metadata.
 */
public record LinkMetadata(
    @JsonProperty("href") String href,
    @JsonProperty("text") String text,
    @JsonProperty("title") @Nullable String title,
    @JsonProperty("link_type") String linkType,
    @JsonProperty("rel") List<String> rel,
    @JsonProperty("attributes") Map<String, String> attributes
) { }
