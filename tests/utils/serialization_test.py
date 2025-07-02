"""Tests for serialization utilities."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any
from unittest.mock import Mock

import pytest

from kreuzberg._utils._serialization import (
    deserialize,
    deserialize_from_str,
    encode_hook,
    serialize,
    serialize_to_str,
)


class Color(Enum):
    """Test enum."""

    RED = "red"
    GREEN = "green"
    BLUE = "blue"


@dataclass
class SampleDataclass:
    """Test dataclass."""

    name: str
    value: int
    color: Color


class SampleError(Exception):
    """Test exception class."""


def test_encode_hook_callable() -> None:
    """Test encoding callable objects returns None."""

    def test_func() -> None:
        pass

    assert encode_hook(test_func) is None
    assert encode_hook(lambda x: x) is None
    assert encode_hook(print) is None


def test_encode_hook_exception() -> None:
    """Test encoding exceptions."""
    exc = ValueError("Test error message")
    result = encode_hook(exc)

    assert result == {"message": "Test error message", "type": "ValueError"}

    # Custom exception
    custom_exc = SampleError("Custom error")
    result = encode_hook(custom_exc)

    assert result == {"message": "Custom error", "type": "SampleError"}


def test_encode_hook_dataclass() -> None:
    """Test encoding dataclasses."""
    obj = SampleDataclass(name="test", value=42, color=Color.RED)
    result = encode_hook(obj)

    assert result == {
        "name": "test",
        "value": 42,
        "color": "red",  # Enum converted to value
    }


def test_encode_hook_dataclass_type() -> None:
    """Test that dataclass types are not encoded."""
    with pytest.raises(TypeError, match="Unsupported type"):
        encode_hook(SampleDataclass)


def test_encode_hook_dict_methods() -> None:
    """Test encoding objects with dict methods."""
    # to_dict
    obj = Mock()
    obj.to_dict.return_value = {"key": "value"}
    assert encode_hook(obj) == {"key": "value"}

    # as_dict
    obj = Mock(spec=[])
    obj.as_dict = Mock(return_value={"key2": "value2"})
    assert encode_hook(obj) == {"key2": "value2"}

    # dict method
    obj = Mock(spec=[])
    obj.dict = Mock(return_value={"key3": "value3"})
    assert encode_hook(obj) == {"key3": "value3"}

    # model_dump (Pydantic v2)
    obj = Mock(spec=[])
    obj.model_dump = Mock(return_value={"key4": "value4"})
    assert encode_hook(obj) == {"key4": "value4"}


def test_encode_hook_list_methods() -> None:
    """Test encoding objects with list methods."""
    # to_list
    obj = Mock(spec=[])
    obj.to_list = Mock(return_value=[1, 2, 3])
    assert encode_hook(obj) == [1, 2, 3]

    # tolist (numpy style)
    obj = Mock(spec=[])
    obj.tolist = Mock(return_value=[4, 5, 6])
    assert encode_hook(obj) == [4, 5, 6]


def test_encode_hook_pil_image() -> None:
    """Test encoding PIL images returns None."""
    mock_image = Mock()
    mock_image.save = Mock()
    mock_image.format = "PNG"

    assert encode_hook(mock_image) is None


def test_encode_hook_pandas_dataframe() -> None:
    """Test encoding pandas DataFrames."""
    mock_df = Mock()
    mock_df.to_dict.return_value = {"col1": [1, 2], "col2": [3, 4]}

    result = encode_hook(mock_df)
    assert result == {"__dataframe__": {"col1": [1, 2], "col2": [3, 4]}}


def test_encode_hook_pandas_dataframe_error() -> None:
    """Test encoding pandas DataFrames with error falls through."""
    mock_df = Mock()
    mock_df.to_dict.side_effect = Exception("DataFrame error")

    with pytest.raises(TypeError, match="Unsupported type"):
        encode_hook(mock_df)


def test_encode_hook_unsupported() -> None:
    """Test encoding unsupported types raises TypeError."""

    class UnsupportedType:
        pass

    obj = UnsupportedType()

    with pytest.raises(TypeError, match="Unsupported type.*UnsupportedType"):
        encode_hook(obj)


def test_serialize_simple() -> None:
    """Test serializing simple types."""
    # String
    result = serialize("hello")
    assert isinstance(result, bytes)

    # Number
    result = serialize(42)
    assert isinstance(result, bytes)

    # List
    result = serialize([1, 2, 3])
    assert isinstance(result, bytes)

    # Dict
    result = serialize({"key": "value"})
    assert isinstance(result, bytes)


def test_serialize_with_kwargs() -> None:
    """Test serializing dict with additional kwargs."""
    base = {"key1": "value1"}
    result = serialize(base, key2="value2", key3=123)

    # Deserialize to verify
    from msgspec import msgpack

    decoded = msgpack.decode(result)

    assert decoded == {"key1": "value1", "key2": "value2", "key3": 123}


def test_serialize_complex_object() -> None:
    """Test serializing complex objects."""
    obj = SampleDataclass(name="test", value=42, color=Color.GREEN)
    result = serialize(obj)

    assert isinstance(result, bytes)

    # Verify it can be decoded
    from msgspec import msgpack

    decoded = msgpack.decode(result)
    assert decoded["name"] == "test"
    assert decoded["value"] == 42
    assert decoded["color"] == "green"


def test_serialize_error() -> None:
    """Test serialization error handling."""

    # Create an object that can't be serialized
    class BadObject:
        def __init__(self) -> None:
            self.circular = self

    obj = BadObject()

    with pytest.raises(ValueError, match="Failed to serialize"):
        serialize(obj)


def test_deserialize_simple() -> None:
    """Test deserializing simple types."""
    # String
    data = serialize("hello")
    result = deserialize(data, str)
    assert result == "hello"

    # Number
    data = serialize(42)
    result = deserialize(data, int)
    assert result == 42

    # List
    data = serialize([1, 2, 3])
    result = deserialize(data, list[int])
    assert result == [1, 2, 3]


def test_deserialize_dict() -> None:
    """Test deserializing dictionaries."""
    data = serialize({"key": "value", "num": 123})
    result = deserialize(data, dict[str, Any])

    assert result == {"key": "value", "num": 123}


def test_deserialize_error() -> None:
    """Test deserialization error handling."""
    data = serialize("not a number")

    with pytest.raises(ValueError, match="Failed to deserialize to int"):
        deserialize(data, int)


def test_serialize_to_str() -> None:
    """Test serializing to string."""
    result = serialize_to_str({"key": "value"})

    assert isinstance(result, str)
    assert len(result) > 0


def test_serialize_to_str_with_kwargs() -> None:
    """Test serializing to string with kwargs."""
    result = serialize_to_str({"base": "value"}, extra="data")

    # Decode to verify
    from msgspec import msgpack

    decoded = msgpack.decode(result.encode("utf-8"))

    assert decoded == {"base": "value", "extra": "data"}


def test_deserialize_from_str() -> None:
    """Test deserializing from string."""
    data_str = serialize_to_str([1, 2, 3])
    result = deserialize_from_str(data_str, list[int])

    assert result == [1, 2, 3]


def test_roundtrip_complex() -> None:
    """Test roundtrip serialization of complex objects."""
    original = {
        "name": "test",
        "items": [1, 2, 3],
        "metadata": {
            "created": "2024-01-01",
            "tags": ["a", "b", "c"],
        },
        "count": 42,
    }

    # Serialize and deserialize
    serialized = serialize(original)
    result = deserialize(serialized, dict[str, Any])

    assert result == original


def test_serialize_none_values() -> None:
    """Test serializing None values."""
    data = {"key": None, "value": 123}
    result = serialize(data)

    # Verify it can be decoded
    from msgspec import msgpack

    decoded = msgpack.decode(result)

    assert decoded["key"] is None
    assert decoded["value"] == 123


def test_encode_hook_method_priority() -> None:
    """Test method priority in encode_hook."""
    # Object with multiple methods - to_dict should be used first
    obj = Mock()
    obj.to_dict = Mock(return_value={"from": "to_dict"})
    obj.as_dict = Mock(return_value={"from": "as_dict"})
    obj.dict = Mock(return_value={"from": "dict"})

    result = encode_hook(obj)
    assert result == {"from": "to_dict"}
    obj.to_dict.assert_called_once()
    obj.as_dict.assert_not_called()
    obj.dict.assert_not_called()


def test_encode_hook_json_method() -> None:
    """Test encoding objects with json method."""
    obj = Mock(spec=[])
    obj.json = Mock(return_value='{"key": "json_value"}')

    result = encode_hook(obj)
    assert result == '{"key": "json_value"}'


def test_serialize_bytes_input() -> None:
    """Test that serialize handles bytes properly."""
    data = b"binary data"
    result = serialize(data)

    assert isinstance(result, bytes)
    # Verify it can be decoded back
    from msgspec import msgpack

    decoded = msgpack.decode(result)
    assert decoded == data


def test_deserialize_with_bytes_input() -> None:
    """Test deserialize with bytes input."""
    original = {"test": "data"}
    serialized = serialize(original)

    # Should work with bytes
    result = deserialize(serialized, dict[str, str])
    assert result == original


def test_deserialize_with_string_input() -> None:
    """Test deserialize with string input."""
    original = {"test": "data"}
    serialized_str = serialize_to_str(original)

    # Should work with string
    result = deserialize(serialized_str, dict[str, str])
    assert result == original
