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
        use crate::schema::SchemaKind;

        match &self.kind {
            SchemaKind::Null => Layout::new(0, 1),
            SchemaKind::Bool => Layout::new(1, 1),
            SchemaKind::Int8 { .. } | SchemaKind::UInt8 { .. } => Layout::new(1, 1),
            SchemaKind::Int16 { .. } | SchemaKind::UInt16 { .. } => Layout::new(2, 2),
            SchemaKind::Int32 { .. } | SchemaKind::UInt32 { .. } | SchemaKind::Float32 { .. } => {
                Layout::new(4, 4)
            }
            SchemaKind::Int64 { .. } | SchemaKind::UInt64 { .. } | SchemaKind::Float64 { .. } => {
                Layout::new(8, 8)
            }

            SchemaKind::String { .. } => Layout::new(0, 8),
            SchemaKind::Bytes { .. } => Layout::new(0, 8),

            SchemaKind::Array { items, .. } => {
                let item_layout = items.layout();
                Layout::new(0, item_layout.align)
            }

            SchemaKind::Object {
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

            SchemaKind::Tuple { items } => {
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

            SchemaKind::Union { any_of } => {
                let max_size = any_of.iter().map(|s| s.layout().size).max().unwrap_or(0);
                let max_align = any_of.iter().map(|s| s.layout().align).max().unwrap_or(1);
                Layout::new(max_size, max_align)
            }

            SchemaKind::Literal { .. } => Layout::new(0, 1),
            SchemaKind::Enum { .. } => Layout::new(1, 1),

            SchemaKind::Ref { .. } => Layout::new(0, 1),
            SchemaKind::Named { schema, .. } => schema.layout(),

            SchemaKind::Function { .. } => Layout::new(0, 1),
            SchemaKind::Void => Layout::new(0, 1),
            SchemaKind::Never => Layout::new(0, 1),
            SchemaKind::Any => Layout::new(0, 8),
            SchemaKind::Unknown => Layout::new(0, 8),
            SchemaKind::Undefined => Layout::new(0, 1),
            SchemaKind::Recursive { schema } => schema.layout(),
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

        assert_eq!(layout.offsets, vec![0, 4, 6]);
        assert_eq!(layout.size, 8);
        assert_eq!(layout.align, 4);
    }
}
