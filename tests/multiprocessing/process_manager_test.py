"""Tests for process pool manager."""

from __future__ import annotations

import multiprocessing as mp
from concurrent.futures import Future, ProcessPoolExecutor
from unittest.mock import patch

import pytest

from kreuzberg._multiprocessing.process_manager import ProcessPoolManager


def simple_function(x: int) -> int:
    """Simple function for testing."""
    return x * 2


def error_function() -> None:
    """Function that raises an error."""
    raise ValueError("Test error")


class TestProcessPoolManager:
    """Tests for ProcessPoolManager class."""

    def test_init_default(self) -> None:
        """Test initialization with default workers."""
        manager = ProcessPoolManager()
        expected_workers = mp.cpu_count()
        assert manager.max_workers == expected_workers
        assert manager._pool is None
        assert manager._active is False

    def test_init_custom_workers(self) -> None:
        """Test initialization with custom workers."""
        manager = ProcessPoolManager(max_workers=4)
        assert manager.max_workers == 4

    def test_init_zero_workers(self) -> None:
        """Test initialization with zero workers defaults to CPU count."""
        manager = ProcessPoolManager(max_workers=0)
        assert manager.max_workers == mp.cpu_count()

    def test_context_manager(self) -> None:
        """Test context manager functionality."""
        manager = ProcessPoolManager(max_workers=2)

        assert manager._pool is None
        assert not manager._active

        with manager:
            assert manager._pool is not None
            assert isinstance(manager._pool, ProcessPoolExecutor)
            assert manager._active

        assert manager._pool is None
        assert not manager._active

    def test_nested_context_error(self) -> None:
        """Test that nested contexts raise error."""
        manager = ProcessPoolManager(max_workers=2)

        with manager:
            with pytest.raises(RuntimeError) as exc_info, manager:
                pass

            assert "already active" in str(exc_info.value)

    def test_submit_success(self) -> None:
        """Test successful task submission."""
        manager = ProcessPoolManager(max_workers=2)

        with manager:
            future = manager.submit(simple_function, 5)
            assert isinstance(future, Future)
            result = future.result()
            assert result == 10

    def test_submit_error(self) -> None:
        """Test task submission that raises error."""
        manager = ProcessPoolManager(max_workers=2)

        with manager:
            future = manager.submit(error_function)
            with pytest.raises(ValueError, match="Test error") as exc_info:
                future.result()
            assert "Test error" in str(exc_info.value)

    def test_submit_without_context(self) -> None:
        """Test submission without active context raises error."""
        manager = ProcessPoolManager(max_workers=2)

        with pytest.raises(RuntimeError) as exc_info:
            manager.submit(simple_function, 5)

        assert "not active" in str(exc_info.value)

    def test_map_success(self) -> None:
        """Test successful map operation."""
        manager = ProcessPoolManager(max_workers=2)

        with manager:
            results = list(manager.map(simple_function, [1, 2, 3, 4, 5]))
            assert results == [2, 4, 6, 8, 10]

    def test_map_without_context(self) -> None:
        """Test map without active context raises error."""
        manager = ProcessPoolManager(max_workers=2)

        with pytest.raises(RuntimeError) as exc_info:
            list(manager.map(simple_function, [1, 2, 3]))

        assert "not active" in str(exc_info.value)

    def test_shutdown(self) -> None:
        """Test explicit shutdown."""
        manager = ProcessPoolManager(max_workers=2)

        with manager:
            assert manager._active
            manager.shutdown()
            assert not manager._active
            assert manager._pool is None

    def test_shutdown_wait_false(self) -> None:
        """Test shutdown with wait=False."""
        manager = ProcessPoolManager(max_workers=2)

        with patch.object(ProcessPoolExecutor, "shutdown") as mock_shutdown, manager:
            manager.shutdown(wait=False)
            mock_shutdown.assert_called_once_with(wait=False)

    def test_shutdown_when_not_active(self) -> None:
        """Test shutdown when pool is not active (should not raise)."""
        manager = ProcessPoolManager(max_workers=2)
        manager.shutdown()  # Should not raise

    def test_is_active(self) -> None:
        """Test is_active property."""
        manager = ProcessPoolManager(max_workers=2)

        assert not manager.is_active

        with manager:
            assert manager.is_active

        assert not manager.is_active

    def test_exception_in_context(self) -> None:
        """Test that pool is cleaned up even with exception."""
        manager = ProcessPoolManager(max_workers=2)

        try:
            with manager:
                assert manager._active
                raise ValueError("Test exception")
        except ValueError:
            pass

        assert not manager._active
        assert manager._pool is None

    def test_multiple_submissions(self) -> None:
        """Test multiple task submissions."""
        manager = ProcessPoolManager(max_workers=2)

        with manager:
            futures = []
            for i in range(10):
                future = manager.submit(simple_function, i)
                futures.append(future)

            results = [f.result() for f in futures]
            expected = [i * 2 for i in range(10)]
            assert results == expected

    def test_submit_kwargs(self) -> None:
        """Test submission with keyword arguments."""

        def func_with_kwargs(x: int, y: int = 10) -> int:
            return x + y

        manager = ProcessPoolManager(max_workers=2)

        with manager:
            future = manager.submit(func_with_kwargs, 5, y=20)
            result = future.result()
            assert result == 25

    def test_map_with_multiple_iterables(self) -> None:
        """Test map with function taking multiple arguments."""

        def add(x: int, y: int) -> int:
            return x + y

        manager = ProcessPoolManager(max_workers=2)

        with manager:
            # Using starmap-like functionality through a wrapper
            inputs = [(1, 2), (3, 4), (5, 6)]
            results = list(manager.map(lambda args: add(*args), inputs))
            assert results == [3, 7, 11]

    def test_worker_initialization(self) -> None:
        """Test that workers are properly initialized."""
        manager = ProcessPoolManager(max_workers=2)

        def get_pid() -> int:
            import os

            return os.getpid()

        with manager:
            # Submit multiple tasks to ensure they run on different processes
            futures = [manager.submit(get_pid) for _ in range(4)]
            pids = [f.result() for f in futures]

            # Should have at most 2 unique PIDs (2 workers)
            unique_pids = set(pids)
            assert len(unique_pids) <= 2
            assert len(unique_pids) >= 1

    def test_cleanup_on_error(self) -> None:
        """Test pool cleanup when error occurs during initialization."""
        manager = ProcessPoolManager(max_workers=2)

        with patch("concurrent.futures.ProcessPoolExecutor") as mock_pool_class:
            mock_pool_class.side_effect = RuntimeError("Pool creation failed")

            with pytest.raises(RuntimeError) as exc_info, manager:
                pass

            assert "Pool creation failed" in str(exc_info.value)
            assert not manager._active
            assert manager._pool is None
