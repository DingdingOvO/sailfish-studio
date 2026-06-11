//! Batch renderer with z-sorting and texture grouping.
//!
//! Collects renderable items and sorts them by z-order (ascending) then by
//! texture ID for efficient batching.

use serde::{Deserialize, Serialize};

// ── Types ────────────────────────────────────────────────────────────────────

/// A single item to be rendered in a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    pub texture_id: u32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub alpha: f64,
}

impl BatchItem {
    pub fn new(texture_id: u32, x: f64, y: f64, z: f64) -> Self {
        Self {
            texture_id,
            x,
            y,
            z,
            width: 1.0,
            height: 1.0,
            rotation: 0.0,
            alpha: 1.0,
        }
    }
}

/// A sorted batch of render items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub items: Vec<BatchItem>,
}

impl Batch {
    pub fn new(items: Vec<BatchItem>) -> Self {
        Self { items }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// Builder for constructing a sorted batch.
#[derive(Debug, Clone, Default)]
pub struct BatchBuilder {
    items: Vec<BatchItem>,
}

impl BatchBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an item to the builder.
    pub fn add(&mut self, item: BatchItem) -> &mut Self {
        self.items.push(item);
        self
    }

    /// Build the batch with items sorted by z then texture_id.
    pub fn build(self) -> Batch {
        Batch {
            items: build_batch(self.items),
        }
    }
}

// ── Sorting ──────────────────────────────────────────────────────────────────

/// Sort items by z (ascending), then by texture_id (ascending) for efficient
/// draw-call batching.
pub fn build_batch(mut items: Vec<BatchItem>) -> Vec<BatchItem> {
    items.sort_by(|a, b| {
        a.z.partial_cmp(&b.z)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.texture_id.cmp(&b.texture_id))
    });
    items
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_by_z_then_texture() {
        let items = vec![
            BatchItem::new(2, 0.0, 0.0, 5.0),
            BatchItem::new(1, 0.0, 0.0, 1.0),
            BatchItem::new(1, 0.0, 0.0, 5.0),
            BatchItem::new(3, 0.0, 0.0, 3.0),
        ];

        let sorted = build_batch(items);

        // z=1 (tex 1), z=3 (tex 3), z=5 (tex 1), z=5 (tex 2)
        assert_eq!(sorted[0].z, 1.0);
        assert_eq!(sorted[0].texture_id, 1);
        assert_eq!(sorted[1].z, 3.0);
        assert_eq!(sorted[1].texture_id, 3);
        assert_eq!(sorted[2].z, 5.0);
        assert_eq!(sorted[2].texture_id, 1);
        assert_eq!(sorted[3].z, 5.0);
        assert_eq!(sorted[3].texture_id, 2);
    }

    #[test]
    fn batch_builder_produces_sorted_batch() {
        let mut builder = BatchBuilder::new();
        builder
            .add(BatchItem::new(1, 10.0, 20.0, 2.0))
            .add(BatchItem::new(2, 30.0, 40.0, 0.0))
            .add(BatchItem::new(3, 50.0, 60.0, 1.0));

        let batch = builder.build();
        assert_eq!(batch.len(), 3);
        assert_eq!(batch.items[0].z, 0.0);
        assert_eq!(batch.items[1].z, 1.0);
        assert_eq!(batch.items[2].z, 2.0);
    }

    #[test]
    fn empty_batch() {
        let batch = BatchBuilder::new().build();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn items_preserve_properties_after_sort() {
        let items = vec![
            BatchItem {
                texture_id: 5,
                x: 100.0,
                y: 200.0,
                z: 10.0,
                width: 50.0,
                height: 60.0,
                rotation: 45.0,
                alpha: 0.5,
            },
        ];
        let sorted = build_batch(items);
        assert_eq!(sorted.len(), 1);
        let item = &sorted[0];
        assert!((item.x - 100.0).abs() < 0.01);
        assert!((item.y - 200.0).abs() < 0.01);
        assert!((item.width - 50.0).abs() < 0.01);
        assert!((item.height - 60.0).abs() < 0.01);
        assert!((item.rotation - 45.0).abs() < 0.01);
        assert!((item.alpha - 0.5).abs() < 0.01);
    }

    #[test]
    fn same_z_sorts_by_texture() {
        let items = vec![
            BatchItem::new(3, 0.0, 0.0, 1.0),
            BatchItem::new(1, 0.0, 0.0, 1.0),
            BatchItem::new(2, 0.0, 0.0, 1.0),
        ];
        let sorted = build_batch(items);
        let tex_ids: Vec<u32> = sorted.iter().map(|i| i.texture_id).collect();
        assert_eq!(tex_ids, vec![1, 2, 3]);
    }
}
