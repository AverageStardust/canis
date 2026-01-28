use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::Display,
    io::Write,
};

use anyhow::Result;
use convert_to_spaces::convert_to_spaces;

use crate::{
    CommandError,
    log::Log,
    processes::assemble::{
        diagnostics::{Annotatable, Annotation, MultiUseWith},
        error::Diagnostic,
        instruction::InstructionParserRegistry,
        line_span::{Span, SpannableIter},
    },
};

mod diagnostics;
mod instruction;
mod line_span;

pub fn assemble(
    log: &mut Log,
    input: &str,
    input_name: impl Display,
    mut output: impl Write,
) -> Result<(), CommandError> {
    let input = convert_to_spaces(input, 4);
    let instructions = InstructionParserRegistry::initialize()?;
    let labels = handle_pass_error(
        log,
        &input,
        input_name,
        "initial",
        label_pass(&input, &instructions),
    )?;
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

fn label_pass<'a>(
    input: &'a str,
    instructions: &InstructionParserRegistry,
) -> Result<HashMap<&'a str, (usize, Span)>, Vec<LabelPassError>> {
    let mut labels = HashMap::new();
    let mut pc: usize = 0;
    let mut pc_poisoned = false;
    let mut errors: Vec<LabelPassError> = Vec::new();

    let mut last_instr: Option<usize> = None;
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
                        diagnostics::LineChar::new(span.line(), span.start()),
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
                                diagnostics::CausalMultiLineSpan::new(cause.into(), span.into()),
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
                diagnostics::LineSpan::new(span.line(), span.start(), span.end()),
            ));
        }
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(labels)
    }
}

#[derive(Debug)]
enum LabelPassError {
    MissingLabel(diagnostics::LineOnly),
    InvalidLabelPrefix(diagnostics::LineChar),
    InvalidLabel(diagnostics::LineSpan),
    DuplicateLabel(diagnostics::CausalMultiLineSpan),
    MissingAssociatedInstruction(diagnostics::LineSpan),

    ParseError(instruction::ParseError),
}

impl From<instruction::ParseError> for LabelPassError {
    fn from(value: instruction::ParseError) -> Self {
        Self::ParseError(value)
    }
}

enum LabelRegistry<'a> {
    Incomplete,
    Complete(HashMap<&'a str, (usize, Span)>),
}

impl<'a> LabelRegistry<'a> {
    fn get_label_location(&self, label: &'a str) -> Option<usize> {
        match self {
            Self::Incomplete => Some(0),
            Self::Complete(registry) => registry.get(label).map(|(loc, _)| *loc),
        }
    }
}

impl error::Diagnostic for LabelPassError {
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

mod error {
    use std::{fmt::Write, usize};

    use crate::processes::assemble::diagnostics::{
        AnnotatedLine, AnnotatedSegment, AnnotationFormat,
    };

    use super::*;
    use anstyle::{Ansi256Color, AnsiColor, Color, Reset, Style};

    pub trait Diagnostic {
        fn code(&self) -> usize;
        fn overview(&self) -> String;
        fn annotation(&self, source: &str) -> Option<Annotation>;
    }

    static RESET: Reset = Reset;
    static CONTEXT: Style = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(103))));
    static SOURCE: Style = Style::new()
        .fg_color(Some(Color::Ansi256(Ansi256Color(146))))
        .bold();
    static COMMENT: Style = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(103))));
    static BOLD: Style = Style::new().bold();
    static ERROR: Style = Style::new()
        .fg_color(Some(Color::Ansi(AnsiColor::Red)))
        .bold();
    static HIGHLIGHT: Style = Style::new()
        .fg_color(Some(Color::Ansi(AnsiColor::BrightBlue)))
        .bold();

    pub fn print_error<D>(input: &str, input_name: impl Display, diagnostic: D)
    where
        D: Diagnostic + std::fmt::Debug,
    {
        let Some(annotation) = diagnostic.annotation(input) else {
            println!(
                "{RESET} failed to print diagnostic, defaulting to debug print: {:?}\n",
                diagnostic
            );
            return;
        };

        print_error_title(diagnostic);

        let width = annotation
            .largest_line()
            .map(|largest| format!("{}", largest + 1).chars().count())
            .unwrap_or(usize::MAX);

        let (start_line, maybe_end) = annotation.line_range();

        let mut buf = String::new();

        if let Some(end_line) = maybe_end {
            writeln!(
                buf,
                "{CONTEXT} {:>width$} ╭[{RESET}{BOLD}{input_name}: {}:{}{RESET}{CONTEXT}]",
                "",
                start_line + 1,
                end_line + 1
            )
            .fmt_expect();
        } else {
            writeln!(
                buf,
                "{CONTEXT} {:>width$} ╭[{RESET}{input_name}: {}{CONTEXT}]{RESET}",
                "",
                start_line + 1
            )
            .fmt_expect();
        }

        for line in annotation.lines() {
            match line {
                AnnotatedLine::Line {
                    line_idx,
                    has_error,
                    segments,
                } => {
                    let format = if has_error { ERROR } else { CONTEXT };
                    write!(buf, " {format}{:>width$}{CONTEXT} │ ", line_idx + 1).fmt_expect();
                    write_segments(&mut buf, segments);
                }
                AnnotatedLine::AnnotationOnly { segments } => {
                    write!(buf, " {:>width$}{CONTEXT} • ", "").fmt_expect();
                    write_segments(&mut buf, segments);
                }
                AnnotatedLine::FlippedOrder => {
                    write!(buf, " {:>width$}{HIGHLIGHT} ⮁ ", "").fmt_expect();
                }
            }

            writeln!(buf, "{RESET}").fmt_expect();
        }
        writeln!(buf, "{CONTEXT} {:─>width$}─╯{RESET}", "").fmt_expect();

        println!("{buf}");
    }

    fn print_error_title<D: Diagnostic>(diagnostic: D) {
        let code = diagnostic.code();
        let overview = diagnostic.overview();
        println!(" {ERROR}[E{code:0>3}]{RESET}{BOLD}: {overview}{RESET}");
    }

    fn write_segments(buf: &mut String, segments: Vec<AnnotatedSegment>) {
        for segment in segments {
            write_segment(buf, segment);
        }
    }

    fn write_segment(buf: &mut String, segment: AnnotatedSegment) {
        let AnnotatedSegment(content, format) = segment;
        write!(buf, "{RESET}").fmt_expect();
        match format {
            AnnotationFormat::Source => write!(buf, "{SOURCE}").fmt_expect(),
            AnnotationFormat::SourceComment => write!(buf, "{COMMENT}").fmt_expect(),
            AnnotationFormat::Text => write!(buf, "{BOLD}").fmt_expect(),
            AnnotationFormat::Error => write!(buf, "{ERROR}").fmt_expect(),
            AnnotationFormat::Highlight => write!(buf, "{HIGHLIGHT}").fmt_expect(),
        }
        write!(buf, "{content}").fmt_expect();
    }

    trait FmtExpect {
        fn fmt_expect(self);
    }

    impl FmtExpect for Result<(), std::fmt::Error> {
        fn fmt_expect(self) {
            if let Err(err) = self {
                panic!(
                    "error formatting to string buffer for error annotation (root cause: {err})"
                );
            }
        }
    }
}
