#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub size: usize,
    pub align: usize,
    pub offsets: Vec<usize>,
}

impl Layout {
    pub fn new(size: usize, align: usize) -> Self {
        Self {
            size,
            align,
            offsets: vec![],
        }
    }

    pub fn with_offsets(size: usize, align: usize, offsets: Vec<usize>) -> Self {
        Self {
            size,
            align,
            offsets,
        }
    }
}

impl crate::schema::Schema {
    pub fn layout(&self) -> Layout {
        use crate::schema::Schema;

        match self {
            Schema::Null => Layout::new(0, 1),
            Schema::Bool => Layout::new(1, 1),
            Schema::Int8 { .. } | Schema::UInt8 { .. } => Layout::new(1, 1),
            Schema::Int16 { .. } | Schema::UInt16 { .. } => Layout::new(2, 2),
            Schema::Int32 { .. } | Schema::UInt32 { .. } | Schema::Float32 { .. } => {
                Layout::new(4, 4)
            }
            Schema::Int64 { .. } | Schema::UInt64 { .. } | Schema::Float64 { .. } => {
                Layout::new(8, 8)
            }

            Schema::String { .. } => Layout::new(0, 8), // Variable size, pointer-aligned
            Schema::Bytes { .. } => Layout::new(0, 8),  // Variable size, pointer-aligned

            Schema::Array { items, .. } => {
                let item_layout = items.layout();
                Layout::new(0, item_layout.align)
            }

            Schema::Object {
                properties,
                required,
                ..
            } => {
                let mut offset = 0;
                let mut max_align = 1;
                let mut offsets = vec![];

                for (name, schema) in properties {
                    if !required.contains(name) {
                        offsets.push(0);
                        continue;
                    }

                    let field_layout = schema.layout();

                    if field_layout.align > 1 {
                        offset = (offset + field_layout.align - 1) & !(field_layout.align - 1);
                    }

                    offsets.push(offset);
                    offset += field_layout.size;
                    max_align = max_align.max(field_layout.align);
                }

                if max_align > 1 {
                    offset = (offset + max_align - 1) & !(max_align - 1);
                }

                Layout::with_offsets(offset, max_align, offsets)
            }

            Schema::Tuple { items } => {
                let mut offset = 0;
                let mut max_align = 1;
                let mut offsets = vec![];

                for item in items {
                    let field_layout = item.layout();

                    if field_layout.align > 1 {
                        offset = (offset + field_layout.align - 1) & !(field_layout.align - 1);
                    }

                    offsets.push(offset);
                    offset += field_layout.size;
                    max_align = max_align.max(field_layout.align);
                }

                if max_align > 1 {
                    offset = (offset + max_align - 1) & !(max_align - 1);
                }

                Layout::with_offsets(offset, max_align, offsets)
            }

            Schema::Union { any_of } => {
                let max_size = any_of.iter().map(|s| s.layout().size).max().unwrap_or(0);
                let max_align = any_of.iter().map(|s| s.layout().align).max().unwrap_or(1);
                Layout::new(max_size, max_align)
            }

            Schema::Literal { .. } => Layout::new(0, 1),
            Schema::Enum { .. } => Layout::new(1, 1),

            Schema::Ref { .. } => Layout::new(0, 1),
            Schema::Named { schema, .. } => schema.layout(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::SchemaBuilder;

    #[test]
    fn test_primitive_layouts() {
        assert_eq!(SchemaBuilder::bool().layout(), Layout::new(1, 1));
        assert_eq!(SchemaBuilder::int8().layout(), Layout::new(1, 1));
        assert_eq!(SchemaBuilder::int16().layout(), Layout::new(2, 2));
        assert_eq!(SchemaBuilder::int32().layout(), Layout::new(4, 4));
        assert_eq!(SchemaBuilder::int64().layout(), Layout::new(8, 8));
        assert_eq!(SchemaBuilder::float64().layout(), Layout::new(8, 8));
    }

    #[test]
    fn test_object_layout() {
        let schema = SchemaBuilder::object()
            .field("a", SchemaBuilder::int8())
            .field("b", SchemaBuilder::int64())
            .field("c", SchemaBuilder::int8())
            .build();

        let layout = schema.layout();

        // a: offset 0
        // b: offset 8 (aligned to 8)
        // c: offset 16
        // total: 24 (padded to 8)
        assert_eq!(layout.offsets, vec![0, 8, 16]);
        assert_eq!(layout.size, 24);
        assert_eq!(layout.align, 8);
    }

    #[test]
    fn test_tuple_layout() {
        let schema = SchemaBuilder::tuple(vec![
            SchemaBuilder::int32(),
            SchemaBuilder::int16(),
            SchemaBuilder::int8(),
        ]);

        let layout = schema.layout();

        // 0: int32 (4 bytes)
        // 4: int16 (2 bytes) - no padding needed
        // 6: int8 (1 byte)
        // total: 7, aligned to 4
        assert_eq!(layout.offsets, vec![0, 4, 6]);
        assert_eq!(layout.size, 8); // padded to alignment
        assert_eq!(layout.align, 4);
    }
}
