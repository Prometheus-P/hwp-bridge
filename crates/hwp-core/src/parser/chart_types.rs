// crates/hwp-core/src/parser/chart_types.rs

#[derive(Clone, Copy)]
pub(crate) enum ChartFieldKind {
    Boolean,
    Integer,
    Long,
    Single,
    Double,
    String,
    Object(&'static str),
}

#[derive(Clone, Copy)]
pub(crate) struct ChartField {
    pub(crate) name: &'static str,
    pub(crate) kind: ChartFieldKind,
}

impl ChartField {
    pub(crate) const fn new(name: &'static str, kind: ChartFieldKind) -> Self {
        Self { name, kind }
    }
}
