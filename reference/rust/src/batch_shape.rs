//! Reference preflight for the full finance HPC batch envelope.
//!
//! The caller must obtain all region lengths from trusted runtime metadata.
//! This validates shape only; it does not prove ownership, provenance or ABI.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchShape {
    pub bar_count: usize,
    pub return_count: usize,
    pub rolling_count: usize,
    pub window: usize,
    pub chunk_size: usize,
    pub chunk_count: usize,
    pub bars_capacity: usize,
    pub returns_capacity: usize,
    pub rolling_capacity: usize,
    pub chunks_capacity: usize,
    pub total_capacity: usize,
    pub signal_capacity: usize,
    pub order_capacity: usize,
    pub decision_capacity: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchShapeError {
    InsufficientBars,
    ReturnCount,
    InvalidWindow,
    RollingCount,
    InvalidChunkSize,
    ChunkCount,
    Capacity(&'static str),
}

/// Validate exact logical dimensions and physical Region capacities.
///
/// A complete finance pipeline must have at least one return and rolling
/// value because `latest_signal_kernel` indexes both tails. The chunk
/// count uses subtraction-before-division to avoid ceiling overflow.
/// This is a pure host reference, not a transfer or compute capability.
pub fn validate_batch_shape(shape: BatchShape) -> Result<usize, BatchShapeError> {
    if shape.bar_count < 2 {
        return Err(BatchShapeError::InsufficientBars);
    }
    let expected_returns = shape.bar_count - 1;
    if shape.return_count != expected_returns {
        return Err(BatchShapeError::ReturnCount);
    }
    if shape.window == 0 || shape.window > shape.return_count {
        return Err(BatchShapeError::InvalidWindow);
    }
    let expected_rolling = shape.return_count - shape.window + 1;
    if shape.rolling_count != expected_rolling {
        return Err(BatchShapeError::RollingCount);
    }
    if shape.chunk_size == 0 {
        return Err(BatchShapeError::InvalidChunkSize);
    }
    let expected_chunks = 1 + (shape.return_count - 1) / shape.chunk_size;
    if shape.chunk_count != expected_chunks {
        return Err(BatchShapeError::ChunkCount);
    }
    for (name, actual, expected) in [
        ("bars", shape.bars_capacity, shape.bar_count),
        ("returns", shape.returns_capacity, shape.return_count),
        ("rolling", shape.rolling_capacity, shape.rolling_count),
        ("chunks", shape.chunks_capacity, shape.chunk_count),
        ("total", shape.total_capacity, 1),
        ("signal", shape.signal_capacity, 1),
        ("order", shape.order_capacity, 1),
        ("decision", shape.decision_capacity, 1),
    ] {
        if actual != expected {
            return Err(BatchShapeError::Capacity(name));
        }
    }
    Ok(expected_chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> BatchShape {
        BatchShape {
            bar_count: 9,
            return_count: 8,
            rolling_count: 6,
            window: 3,
            chunk_size: 3,
            chunk_count: 3,
            bars_capacity: 9,
            returns_capacity: 8,
            rolling_capacity: 6,
            chunks_capacity: 3,
            total_capacity: 1,
            signal_capacity: 1,
            order_capacity: 1,
            decision_capacity: 1,
        }
    }

    #[test]
    fn accepts_exact_layout_and_rejects_empty_pipeline() {
        assert_eq!(validate_batch_shape(valid()), Ok(3));
        assert_eq!(
            validate_batch_shape(BatchShape {
                bar_count: 1,
                ..valid()
            }),
            Err(BatchShapeError::InsufficientBars)
        );
    }

    #[test]
    fn rejects_invalid_return_rolling_window_and_chunk_geometry() {
        let cases = [
            (
                BatchShape {
                    return_count: 9,
                    ..valid()
                },
                BatchShapeError::ReturnCount,
            ),
            (
                BatchShape {
                    window: 0,
                    ..valid()
                },
                BatchShapeError::InvalidWindow,
            ),
            (
                BatchShape {
                    window: 9,
                    ..valid()
                },
                BatchShapeError::InvalidWindow,
            ),
            (
                BatchShape {
                    rolling_count: 7,
                    ..valid()
                },
                BatchShapeError::RollingCount,
            ),
            (
                BatchShape {
                    chunk_size: 0,
                    ..valid()
                },
                BatchShapeError::InvalidChunkSize,
            ),
            (
                BatchShape {
                    chunk_count: 2,
                    ..valid()
                },
                BatchShapeError::ChunkCount,
            ),
        ];
        for (shape, expected) in cases {
            assert_eq!(validate_batch_shape(shape), Err(expected));
        }
    }

    #[test]
    fn rejects_malformed_physical_extents_before_compute() {
        let cases = [
            (
                BatchShape {
                    bars_capacity: 8,
                    ..valid()
                },
                "bars",
            ),
            (
                BatchShape {
                    returns_capacity: 7,
                    ..valid()
                },
                "returns",
            ),
            (
                BatchShape {
                    rolling_capacity: 5,
                    ..valid()
                },
                "rolling",
            ),
            (
                BatchShape {
                    chunks_capacity: 2,
                    ..valid()
                },
                "chunks",
            ),
            (
                BatchShape {
                    total_capacity: 0,
                    ..valid()
                },
                "total",
            ),
            (
                BatchShape {
                    signal_capacity: 2,
                    ..valid()
                },
                "signal",
            ),
            (
                BatchShape {
                    order_capacity: 0,
                    ..valid()
                },
                "order",
            ),
            (
                BatchShape {
                    decision_capacity: 2,
                    ..valid()
                },
                "decision",
            ),
        ];
        for (shape, name) in cases {
            assert_eq!(
                validate_batch_shape(shape),
                Err(BatchShapeError::Capacity(name))
            );
        }
    }

    #[test]
    fn ceiling_math_does_not_overflow_near_usize_max() {
        let max = usize::MAX;
        let shape = BatchShape {
            bar_count: max,
            return_count: max - 1,
            rolling_count: 1,
            window: max - 1,
            chunk_size: max,
            chunk_count: 1,
            bars_capacity: max,
            returns_capacity: max - 1,
            rolling_capacity: 1,
            chunks_capacity: 1,
            total_capacity: 1,
            signal_capacity: 1,
            order_capacity: 1,
            decision_capacity: 1,
        };
        assert_eq!(validate_batch_shape(shape), Ok(1));
    }

    #[test]
    fn last_chunk_indices_never_exceed_count_after_validation() {
        for count in 1usize..256 {
            for size in 1usize..128 {
                let chunks = 1 + (count - 1) / size;
                let last_start = (chunks - 1) * size;
                assert!(last_start < count);
                for offset in 0..size.min(count - last_start) {
                    assert!(last_start + offset < count);
                }
            }
        }
    }
}
