use std::{
    fmt::{Display, Write},
    usize,
};

use anstyle::{Ansi256Color, AnsiColor, Color, Reset, Style};

use crate::assemble::error::annotations::{
    AnnotatedLine, AnnotatedSegment, Annotation, AnnotationFormat,
};

pub(crate) mod annotations;

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
            panic!("error formatting to string buffer for error annotation (root cause: {err})");
        }
    }
}
