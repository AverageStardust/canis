use crate::instruction::{parser::ParserTypes, registry::InstructionParser};

pub(super) struct Instruction {
    pub(super) name: &'static str,
    pub(super) parser: InstructionParser,

    pub(super) meta: InstructionMeta,
}

pub struct InstructionMeta {
    pub name: &'static str,
    pub details: Option<InstructionMetaDetails>,
}

pub struct InstructionMetaDetails {
    pub display_name: &'static str,
    pub short_description: &'static str,
    explain: Option<fn() -> InstructionMetaExplain>,
}

pub struct InstructionMetaExplain {
    variants: Vec<InstructionMetaExplainVariant>,
}

pub struct InstructionMetaExplainVariant {
    pub args: Vec<(&'static str, ParserTypes)>,
    pub description: &'static str,
}

impl Instruction {
    pub(super) const fn new(name: &'static str, parser: InstructionParser) -> Self {
        Self {
            name,
            parser,
            meta: InstructionMeta {
                name,
                details: None,
            },
        }
    }

    pub(super) const fn new_with_meta(
        name: &'static str,
        parser: InstructionParser,
        display_name: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            name,
            parser,
            meta: InstructionMeta {
                name,
                details: Some(InstructionMetaDetails {
                    display_name,
                    short_description: description,
                    explain: None,
                }),
            },
        }
    }

    pub(super) const fn new_with_explain(
        name: &'static str,
        parser: InstructionParser,
        display_name: &'static str,
        description: &'static str,
        explain: fn() -> InstructionMetaExplain,
    ) -> Self {
        Self {
            name,
            parser,
            meta: InstructionMeta {
                name,
                details: Some(InstructionMetaDetails {
                    display_name,
                    short_description: description,
                    explain: Some(explain),
                }),
            },
        }
    }
}

impl PartialEq for InstructionMeta {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
    }
}

impl PartialOrd for InstructionMeta {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(other.name)
    }
}

impl Eq for InstructionMeta {}

impl Ord for InstructionMeta {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(other.name)
    }
}

impl InstructionMeta {
    pub fn explain(&self) -> Option<InstructionMetaExplain> {
        self.details
            .as_ref()
            .and_then(|details| details.explain)
            .map(|explain| explain())
    }
}

impl InstructionMetaExplain {
    pub(crate) fn new(initial_variant: InstructionMetaExplainVariant) -> Self {
        Self {
            variants: vec![initial_variant],
        }
    }

    pub(crate) fn variant(self, variant: InstructionMetaExplainVariant) -> Self {
        let mut variants = self.variants;
        variants.push(variant);
        Self { variants }
    }
}

impl IntoIterator for InstructionMetaExplain {
    type Item = InstructionMetaExplainVariant;
    type IntoIter = std::vec::IntoIter<InstructionMetaExplainVariant>;
    fn into_iter(self) -> Self::IntoIter {
        self.variants.into_iter()
    }
}

impl InstructionMetaExplainVariant {
    pub(crate) fn new(description: &'static str) -> Self {
        Self {
            args: Vec::new(),
            description,
        }
    }

    pub(crate) fn arg(self, name: &'static str, types: ParserTypes) -> Self {
        let mut args = self.args;
        args.push((name, types));
        Self {
            args,
            description: self.description,
        }
    }
}
