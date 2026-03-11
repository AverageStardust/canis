use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::Display,
};

use anyhow::{Context, Result};
use convert_to_spaces::convert_to_spaces;

use crate::{
    CommandError,
    assemble::{
        error::{
            self, Diagnostic,
            annotations::{self, Annotatable, Annotation, MultiUseWith},
        },
        labels::LabelRegistry,
        line_span::{Span, SpannableIter},
    },
    instruction::{
        parser::ParseError, raw::RawInstruction, registry::InstructionParserRegistry,
        types::Location,
    },
    log::Log,
};

pub fn assemble(
    log: &mut Log,
    input: &str,
    input_name: impl Display,
    output: &mut impl std::io::Write,
) -> Result<(), CommandError> {
    let input = convert_to_spaces(input, 4);
    let instructions = InstructionParserRegistry::initialize()?;
    let labels = handle_pass_error(
        log,
        &input,
        &input_name,
        "initial",
        label_pass(&input, &instructions),
    )?;
    let instructions = handle_pass_error(
        log,
        &input,
        &input_name,
        "assembly",
        instr_pass(&input, &instructions, labels),
    )?;
    write_output(output, instructions)?;
    Ok(())
}

fn handle_pass_error<T, D>(
    log: &mut Log,
    source: &str,
    source_name: impl Display,
    pass_name: &'static str,
    res: Result<T, Vec<D>>,
) -> Result<T, CommandError>
where
    D: Diagnostic + std::fmt::Debug,
{
    match res {
        Ok(val) => Ok(val),
        Err(errors) => {
            let len = errors.len();
            for err in errors {
                error::print_error(source, &source_name, err);
            }
            log.error(format!(
                "assembly failed during {pass_name} pass due to {len} previous errors"
            ));
            Err(CommandError::InternallyPrinted)
        }
    }
}

fn write_output(
    output: &mut impl std::io::Write,
    instructions: Vec<RawInstruction>,
) -> Result<(), CommandError> {
    writeln!(output, "v2.0 raw").context("output writing failed while writing header")?;
    let mut word = 0;
    for (idx, instr) in instructions.iter().enumerate() {
        match instr {
            RawInstruction::Small(val) => {
                writeln!(output, "{val:x}").with_context(|| {
                    format!(
                        "output writing failed while writing instruction {idx} as word {}",
                        word
                    )
                })?;
                word += 1;
            }
            RawInstruction::Large(val) => {
                writeln!(output, "{:x}", *val as u16).with_context(|| {
                    format!(
                        "output writing failed while writing instruction {idx} as word {}",
                        word
                    )
                })?;
                word += 1;
                writeln!(output, "{:x}", (val >> 16) as u16).with_context(|| {
                    format!(
                        "output writing failed while writing instruction {idx} as word {}",
                        word
                    )
                })?;
                word += 1;
            }
        }
    }
    Ok(())
}

fn label_pass<'a>(
    input: &'a str,
    instructions: &InstructionParserRegistry,
) -> Result<HashMap<&'a str, (Location, Span)>, Vec<LabelPassError>> {
    let mut labels = HashMap::new();
    let mut pc = Location::new();
    let mut pc_poisoned = false;
    let mut errors: Vec<LabelPassError> = Vec::new();

    let mut last_instr: Option<Location> = None;
    for line in input.lines().spanned() {
        // Ignore everything past comment
        let line = if let Some((pre_comment, _comment)) = line.split_once('#') {
            pre_comment
        } else {
            line
        };

        // Try to pull out a label if there is one, and an instruction if there is one
        let maybe_instr = if let Some((label, remainder)) = line.split_once(':') {
            let label = label.trim();
            let label_str = label.as_str();
            let mut chars = label_str.chars();
            if let Some(prefix) = chars.next() {
                if !prefix.is_ascii_alphabetic() && prefix != '_' {
                    let span = label.as_span();
                    errors.push(LabelPassError::InvalidLabelPrefix(
                        annotations::LineChar::new(span.line(), span.start()),
                    ));
                } else if chars.any(|ch| !ch.is_ascii_alphanumeric() && ch != '_') {
                    let span = label.as_span();
                    errors.push(LabelPassError::InvalidLabel(span.into()));
                } else {
                    match labels.entry(label.as_str()) {
                        Entry::Vacant(vacant) => {
                            vacant.insert((pc, label.as_span()));
                        }
                        Entry::Occupied(occupied) => {
                            let (_, cause) = occupied.remove();
                            let span = label.as_span();
                            errors.push(LabelPassError::DuplicateLabel(
                                annotations::CausalMultiLineSpan::new(cause.into(), span.into()),
                            ));
                        }
                    }
                }
            } else {
                errors.push(LabelPassError::MissingLabel(label.as_span().into()));
            }

            remainder
        } else {
            line
        };

        match instructions.parse_instruction(&LabelRegistry::Incomplete, pc, maybe_instr) {
            Ok(instr_vec) => {
                let size = instr_vec.get_size();
                if size > 0 {
                    last_instr = Some(pc);
                    pc += instr_vec.get_size();
                }
            }
            Err(err) => {
                pc_poisoned = true;
                errors.push(err.into())
            }
        }
    }

    if !pc_poisoned {
        let mut missing_instruction = if let Some(last_pc) = last_instr {
            labels
                .values()
                .filter(|(label_pc, _span)| *label_pc > last_pc)
                .collect::<Vec<_>>()
        } else {
            labels.values().collect::<Vec<_>>()
        };

        missing_instruction.sort_by_key(|(_pc, span)| span.line());

        for (_pc, span) in missing_instruction {
            errors.push(LabelPassError::MissingAssociatedInstruction(
                annotations::LineSpan::new(span.line(), span.start(), span.end()),
            ));
        }
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(labels)
    }
}

fn instr_pass<'a>(
    input: &'a str,
    instructions: &InstructionParserRegistry,
    labels: HashMap<&'a str, (Location, Span)>,
) -> Result<Vec<RawInstruction>, Vec<InstrPassError>> {
    let registry = LabelRegistry::Complete(labels);
    let mut pc = Location::new();
    // PC poisoning can safely be ignored here. If an error occurs, no writes will be made anywhere.
    let mut errors: Vec<InstrPassError> = Vec::new();

    let mut output = Vec::new();

    for line in input.lines().spanned() {
        // Ignore everything past comment
        let line = if let Some((pre_comment, _comment)) = line.split_once('#') {
            pre_comment
        } else {
            line
        };

        // Try to pull out a label if there is one, and an instruction if there is one
        // Since label problems are handled in the label pass, this can safely be assumed to be correct
        let maybe_instr = if let Some((_label, remainder)) = line.split_once(':') {
            remainder
        } else {
            line
        };

        match instructions.parse_instruction(&registry, pc, maybe_instr) {
            Ok(instr_vec) => {
                let size = instr_vec.get_size();
                if size > 0 {
                    pc += instr_vec.get_size();
                }
                instr_vec.into_iter().for_each(|instr| output.push(instr));
            }
            Err(err) => errors.push(err.into()),
        }
    }

    // Labels missing instructions are handled above, they are can safely be assumed as handled here

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(output)
    }
}

#[derive(Debug)]
enum LabelPassError {
    MissingLabel(annotations::LineOnly),
    InvalidLabelPrefix(annotations::LineChar),
    InvalidLabel(annotations::LineSpan),
    DuplicateLabel(annotations::CausalMultiLineSpan),
    MissingAssociatedInstruction(annotations::LineSpan),

    ParseError(ParseError),
}

impl From<ParseError> for LabelPassError {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}

impl Diagnostic for LabelPassError {
    fn code(&self) -> usize {
        match self {
            Self::MissingLabel(_) => 100,
            Self::InvalidLabelPrefix(_) => 101,
            Self::InvalidLabel(_) => 102,
            Self::DuplicateLabel(_) => 103,
            Self::MissingAssociatedInstruction(_) => 104,

            Self::ParseError(err) => err.code(),
        }
    }

    fn overview(&self) -> String {
        match self {
            Self::MissingLabel(_) => "missing label",
            Self::InvalidLabelPrefix(_) => "invalid label prefix",
            Self::InvalidLabel(_) => "invalid label",
            Self::DuplicateLabel(_) => "duplicate label",
            Self::MissingAssociatedInstruction(_) => "missing an associated instruction",

            Self::ParseError(err) => return err.overview(),
        }
        .to_string()
    }

    fn annotation(&self, source: &str) -> Option<Annotation> {
        match self {
            Self::MissingLabel(span) => span.annotate(
                source,
                ["A label is expected here"],
                Some(
                    "The character ':' is specifically used to suffix labels.
                    If it is present, a label will be expected on that line.",
                ),
            ),
            Self::InvalidLabelPrefix(span) => span.annotate(
                source,
                [|prefix| format!("Prefix \"{prefix}\" is not permitted")],
                Some(
                    "Labels must begin with an ascii
                alphabetic character, or an underscore.",
                ),
            ),
            Self::InvalidLabel(span) => span.annotate(
                source,
                [|label| format!("Label \"{label}\" is not permitted")],
                Some(
                    "Labels must only contain ascii
                alphanumeric characters, and underscores.",
                ),
            ),
            Self::DuplicateLabel(span) => span.annotate(
                source,
                [
                    MultiUseWith::A(|label| format!("Label \"{label}\" first defined here")),
                    MultiUseWith::B(|label| format!("Tries to redefine \"{label}\" here")),
                ],
                None,
            ),
            Self::MissingAssociatedInstruction(span) => span.annotate(
                source,
                [|label| {
                    format!("Tried to add label \"{label}\" without an associated instruction")
                }],
                Some(
                    "Labels must be followed by an instruction, either
                on the same line, or on a following line.",
                ),
            ),

            Self::ParseError(err) => err.annotation(source),
        }
    }
}

#[derive(Debug)]
enum InstrPassError {
    ParseError(ParseError),
}

impl From<ParseError> for InstrPassError {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}

impl Diagnostic for InstrPassError {
    fn code(&self) -> usize {
        match self {
            Self::ParseError(err) => err.code(),
        }
    }

    fn overview(&self) -> String {
        match self {
            Self::ParseError(err) => err.overview(),
        }
    }

    fn annotation(&self, source: &str) -> Option<Annotation> {
        match self {
            Self::ParseError(err) => err.annotation(source),
        }
    }
}
