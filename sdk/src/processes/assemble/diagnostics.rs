use std::iter::repeat_n;

use crate::processes::assemble::line_span::Span;

#[derive(PartialEq)]
pub enum AnnotationFormat {
    Source,
    SourceComment,
    Text,
    Error,
    Highlight,
}

pub struct AnnotatedSegment(pub String, pub AnnotationFormat);

impl AnnotatedSegment {
    fn new(content: impl ToString, format: AnnotationFormat) -> Self {
        Self(content.to_string(), format)
    }

    fn source(content: impl ToString) -> Self {
        Self::new(content, AnnotationFormat::Source)
    }

    fn text(content: impl ToString) -> Self {
        Self::new(content, AnnotationFormat::Text)
    }

    fn error(content: impl ToString) -> Self {
        Self::new(content, AnnotationFormat::Error)
    }

    fn highlight(content: impl ToString) -> Self {
        Self::new(content, AnnotationFormat::Highlight)
    }
}

pub enum AnnotatedLine {
    Line {
        line_idx: usize,
        has_error: bool,
        segments: Vec<AnnotatedSegment>,
    },
    AnnotationOnly {
        segments: Vec<AnnotatedSegment>,
    },
    FlippedOrder,
}

impl AnnotatedLine {
    fn auto_comment(self) -> Self {
        match self {
            Self::Line {
                line_idx,
                has_error,
                segments,
            } => {
                let found_comment = false;
                let mut new_segments = vec![];
                for segment in segments {
                    if found_comment && segment.1 == AnnotationFormat::Source {
                        new_segments.push(AnnotatedSegment::new(
                            segment.0,
                            AnnotationFormat::SourceComment,
                        ));
                    } else if let Some((before, after)) = segment.0.split_once('#') {
                        new_segments.push(AnnotatedSegment::new(before, segment.1));
                        new_segments.push(AnnotatedSegment::new(
                            format!("#{after}"),
                            AnnotationFormat::SourceComment,
                        ));
                    } else {
                        new_segments.push(segment);
                    }
                }
                Self::Line {
                    line_idx,
                    has_error,
                    segments: new_segments,
                }
            }
            other => other,
        }
    }

    fn line(line_idx: usize, segments: impl Into<Vec<AnnotatedSegment>>) -> Self {
        Self::Line {
            line_idx,
            has_error: false,
            segments: segments.into(),
        }
        .auto_comment()
    }

    fn line_with_error(line_idx: usize, segments: impl Into<Vec<AnnotatedSegment>>) -> Self {
        Self::Line {
            line_idx,
            has_error: true,
            segments: segments.into(),
        }
        .auto_comment()
    }

    fn annotation(segments: impl Into<Vec<AnnotatedSegment>>) -> Self {
        Self::AnnotationOnly {
            segments: segments.into(),
        }
    }

    fn flipped() -> Self {
        Self::FlippedOrder
    }
}

pub struct Annotation {
    lines: Vec<AnnotatedLine>,
    start: usize,
    end: Option<usize>,
}

impl Annotation {
    fn single_line(annotations: Vec<AnnotatedLine>, line: usize) -> Self {
        Self {
            lines: annotations,
            start: line,
            end: None,
        }
    }

    fn multi_line(annotations: Vec<AnnotatedLine>, start_line: usize, end_line: usize) -> Self {
        Self {
            lines: annotations,
            start: start_line,
            end: Some(end_line),
        }
    }

    pub fn line_range(&self) -> (usize, Option<usize>) {
        (self.start, self.end.clone())
    }

    pub fn largest_line(&self) -> Option<usize> {
        self.lines
            .iter()
            .filter_map(|ann| match ann {
                AnnotatedLine::Line { line_idx, .. } => Some(line_idx),
                AnnotatedLine::AnnotationOnly { .. } => None,
                AnnotatedLine::FlippedOrder => None,
            })
            .max()
            .map(|size| *size)
    }
    pub fn lines(self) -> std::vec::IntoIter<AnnotatedLine> {
        self.lines.into_iter()
    }
}

pub trait UseWith {
    fn use_with(self, with: String) -> String;
}

impl UseWith for () {
    fn use_with(self, _with: String) -> String {
        "".to_string()
    }
}

impl<F: Fn(String) -> String> UseWith for F {
    fn use_with(self, with: String) -> String {
        self(with)
    }
}

impl UseWith for String {
    fn use_with(self, _with: String) -> String {
        self
    }
}

impl UseWith for &str {
    fn use_with(self, _with: String) -> String {
        self.to_string()
    }
}

impl<U: UseWith> UseWith for [U; 1] {
    fn use_with(self, with: String) -> String {
        let [val] = self;
        val.use_with(with)
    }
}

pub enum MultiUseWith<A, B> {
    A(A),
    B(B),
}

impl<A, B> UseWith for MultiUseWith<A, B>
where
    A: UseWith,
    B: UseWith,
{
    fn use_with(self, with: String) -> String {
        match self {
            Self::A(a) => a.use_with(with),
            Self::B(b) => b.use_with(with),
        }
    }
}

pub trait Annotatable<const N: usize = 1> {
    fn annotate(
        &self,
        source: &str,
        annotation: [impl UseWith; N],
        note: Option<&'static str>,
    ) -> Option<Annotation>;
}

#[derive(Debug)]
pub struct LineOnly {
    line_idx: usize,
}

impl LineOnly {
    pub fn new(line: usize) -> Self {
        Self { line_idx: line }
    }
}

impl From<Span> for LineOnly {
    fn from(value: Span) -> Self {
        Self {
            line_idx: value.line(),
        }
    }
}

impl Annotatable<1> for LineOnly {
    fn annotate(
        &self,
        source: &str,
        annotation: [impl UseWith; 1],
        note: Option<&'static str>,
    ) -> Option<Annotation> {
        let (before, line, after) = find_line_and_context(source, self.line_idx)?;
        let mut annotations = vec![];

        if let Some(before) = before {
            annotations.push(AnnotatedLine::line(
                self.line_idx - 1,
                AnnotatedSegment::source(before),
            ));
        }

        let chars = line.chars().count();

        annotations.push(AnnotatedLine::line_with_error(
            self.line_idx,
            AnnotatedSegment::error(&line),
        ));

        if chars > 0 {
            let chars = chars - 1;
            let left = chars.div_ceil(2).min(2);
            let right = chars - left;
            annotations.push(AnnotatedLine::annotation(AnnotatedSegment::error(format!(
                "{}┬{}",
                repeat_n('─', left).collect::<String>(),
                repeat_n('─', right).collect::<String>()
            ))));

            annotations.push(AnnotatedLine::annotation(
                AnnotatedSegment::error(format!("{}╰─ ", repeat_n(' ', left).collect::<String>()))
                    .with(AnnotatedSegment::text(
                        annotation.use_with(line.to_string()),
                    )),
            ));
        } else {
            annotations.push(AnnotatedLine::annotation(AnnotatedSegment::text(
                annotation.use_with(line.to_string()),
            )));
        }

        if let Some(after) = after {
            annotations.push(AnnotatedLine::line(
                self.line_idx + 1,
                AnnotatedSegment::source(after),
            ));
        }

        if let Some(note) = note {
            annotations.push(AnnotatedLine::annotation(vec![]));
            annotations.push(AnnotatedLine::annotation(
                AnnotatedSegment::highlight(format!(" NOTE: ")).with(AnnotatedSegment::text(note)),
            ));
        }

        Some(Annotation::single_line(annotations, self.line_idx))
    }
}

#[derive(Debug)]
pub struct LineChar {
    line_idx: usize,
    char: usize,
}

impl LineChar {
    pub fn new(line: usize, char: usize) -> Self {
        Self {
            line_idx: line,
            char,
        }
    }
}

impl Annotatable<1> for LineChar {
    fn annotate(
        &self,
        source: &str,
        annotation: [impl UseWith; 1],
        note: Option<&'static str>,
    ) -> Option<Annotation> {
        let (before, line, after) = find_line_and_context(source, self.line_idx)?;
        let mut annotations = vec![];

        if let Some(before) = before {
            annotations.push(AnnotatedLine::line(
                self.line_idx - 1,
                AnnotatedSegment::source(before),
            ));
        }

        let (before_span, span, after_span) = extract_span(&line, self.char, self.char + 1);

        annotations.push(AnnotatedLine::line_with_error(
            self.line_idx,
            AnnotatedSegment::source(before_span)
                .with(AnnotatedSegment::error(&span))
                .with(AnnotatedSegment::source(after_span)),
        ));

        annotations.push(AnnotatedLine::annotation(AnnotatedSegment::error(format!(
            "{}┬",
            repeat_n(' ', self.char).collect::<String>()
        ))));

        annotations.push(AnnotatedLine::annotation(
            AnnotatedSegment::error(format!(
                "{}╰─ ",
                repeat_n(' ', self.char).collect::<String>()
            ))
            .with(AnnotatedSegment::text(annotation.use_with(span))),
        ));

        if let Some(after) = after {
            annotations.push(AnnotatedLine::line(
                self.line_idx + 1,
                AnnotatedSegment::source(after),
            ));
        }

        if let Some(note) = note {
            annotations.push(AnnotatedLine::annotation(vec![]));
            annotations.push(AnnotatedLine::annotation(
                AnnotatedSegment::highlight(format!(" NOTE: ")).with(AnnotatedSegment::text(note)),
            ));
        }

        Some(Annotation::single_line(annotations, self.line_idx))
    }
}

#[derive(Debug)]
pub struct LineSpan {
    line_idx: usize,
    start_char: usize,
    end_char: usize,
}

impl LineSpan {
    pub fn new(line: usize, start: usize, end: usize) -> Self {
        assert!(end > start);
        Self {
            line_idx: line,
            start_char: start,
            end_char: end,
        }
    }
}

impl From<Span> for LineSpan {
    fn from(value: Span) -> Self {
        Self {
            line_idx: value.line(),
            start_char: value.start(),
            end_char: value.end(),
        }
    }
}

impl Annotatable<1> for LineSpan {
    fn annotate(
        &self,
        source: &str,
        annotation: [impl UseWith; 1],
        note: Option<&'static str>,
    ) -> Option<Annotation> {
        let (before, line, after) = find_line_and_context(source, self.line_idx)?;
        let mut annotations = vec![];

        if let Some(before) = before {
            annotations.push(AnnotatedLine::line(
                self.line_idx - 1,
                AnnotatedSegment::source(before),
            ));
        }

        let (before_span, span, after_span) = extract_span(&line, self.start_char, self.end_char);

        annotations.push(AnnotatedLine::line_with_error(
            self.line_idx,
            AnnotatedSegment::source(before_span)
                .with(AnnotatedSegment::error(&span))
                .with(AnnotatedSegment::source(after_span)),
        ));

        let chars = self.end_char - self.start_char - 1;
        let left = chars.div_ceil(2).min(2);
        let right = chars - left;

        annotations.push(AnnotatedLine::annotation(AnnotatedSegment::error(format!(
            "{}{}┬{}",
            repeat_n(' ', self.start_char).collect::<String>(),
            repeat_n('─', left).collect::<String>(),
            repeat_n('─', right).collect::<String>()
        ))));

        annotations.push(AnnotatedLine::annotation(
            AnnotatedSegment::error(format!(
                "{}╰─ ",
                repeat_n(' ', self.start_char + left).collect::<String>(),
            ))
            .with(AnnotatedSegment::text(annotation.use_with(span))),
        ));

        if let Some(after) = after {
            annotations.push(AnnotatedLine::line(
                self.line_idx + 1,
                AnnotatedSegment::source(after),
            ));
        }

        if let Some(note) = note {
            annotations.push(AnnotatedLine::annotation(vec![]));
            annotations.push(AnnotatedLine::annotation(
                AnnotatedSegment::highlight(format!(" NOTE: ")).with(AnnotatedSegment::text(note)),
            ));
        }

        Some(Annotation::single_line(annotations, self.line_idx))
    }
}

#[derive(Debug)]
pub struct CausalMultiLineSpan {
    cause: LineSpan,
    effect: LineSpan,
}

impl CausalMultiLineSpan {
    pub fn new(cause: LineSpan, effect: LineSpan) -> Self {
        assert!(cause.line_idx != effect.line_idx);
        Self { cause, effect }
    }
}

impl Annotatable<2> for CausalMultiLineSpan {
    fn annotate(
        &self,
        source: &str,
        annotation: [impl UseWith; 2],
        note: Option<&'static str>,
    ) -> Option<Annotation> {
        let (before_cause, cause, after_cause) =
            find_line_and_context(source, self.cause.line_idx)?;
        let (before_effect, effect, after_effect) =
            find_line_and_context(source, self.effect.line_idx)?;
        let mut annotations = vec![];

        let [cause_annotation, effect_annotation] = annotation;

        if let Some(before) = before_cause {
            annotations.push(AnnotatedLine::line(
                self.cause.line_idx - 1,
                AnnotatedSegment::source(before),
            ));
        }

        let (before_span, span, after_span) =
            extract_span(&cause, self.cause.start_char, self.cause.end_char);

        annotations.push(AnnotatedLine::line(
            self.cause.line_idx,
            AnnotatedSegment::source(before_span)
                .with(AnnotatedSegment::highlight(&span))
                .with(AnnotatedSegment::source(after_span)),
        ));

        let chars = self.cause.end_char - self.cause.start_char - 1;
        let left = chars.div_ceil(2).min(2);
        let right = chars - left;

        annotations.push(AnnotatedLine::annotation(AnnotatedSegment::highlight(
            format!(
                "{}{}┬{}",
                repeat_n(' ', self.cause.start_char).collect::<String>(),
                repeat_n('─', left).collect::<String>(),
                repeat_n('─', right).collect::<String>()
            ),
        )));

        annotations.push(AnnotatedLine::annotation(
            AnnotatedSegment::highlight(format!(
                "{}╰─ ",
                repeat_n(' ', self.cause.start_char + left).collect::<String>(),
            ))
            .with(AnnotatedSegment::text(cause_annotation.use_with(span))),
        ));

        if self.cause.line_idx + 2 < self.effect.line_idx
            || self.cause.line_idx > self.effect.line_idx
        {
            if let Some(after) = after_cause {
                annotations.push(AnnotatedLine::line(
                    self.cause.line_idx + 1,
                    AnnotatedSegment::source(after),
                ));
            }
        }

        if self.cause.line_idx + 3 < self.effect.line_idx {
            annotations.push(AnnotatedLine::annotation(vec![]));
        } else if self.cause.line_idx > self.effect.line_idx {
            annotations.push(AnnotatedLine::flipped());
        }

        if self.cause.line_idx + 1 != self.effect.line_idx
            || self.cause.line_idx > self.effect.line_idx
        {
            if let Some(before) = before_effect {
                annotations.push(AnnotatedLine::line(
                    self.effect.line_idx - 1,
                    AnnotatedSegment::source(before),
                ));
            }
        }

        let (before_span, span, after_span) =
            extract_span(&effect, self.effect.start_char, self.effect.end_char);

        annotations.push(AnnotatedLine::line_with_error(
            self.effect.line_idx,
            AnnotatedSegment::source(before_span)
                .with(AnnotatedSegment::error(&span))
                .with(AnnotatedSegment::source(after_span)),
        ));

        let chars = self.effect.end_char - self.effect.start_char - 1;
        let left = chars.div_ceil(2).min(2);
        let right = chars - left;

        annotations.push(AnnotatedLine::annotation(AnnotatedSegment::error(format!(
            "{}{}┬{}",
            repeat_n(' ', self.effect.start_char).collect::<String>(),
            repeat_n('─', left).collect::<String>(),
            repeat_n('─', right).collect::<String>()
        ))));

        annotations.push(AnnotatedLine::annotation(
            AnnotatedSegment::error(format!(
                "{}╰─ ",
                repeat_n(' ', self.effect.start_char + left).collect::<String>(),
            ))
            .with(AnnotatedSegment::text(effect_annotation.use_with(span))),
        ));

        if let Some(after) = after_effect {
            annotations.push(AnnotatedLine::line(
                self.effect.line_idx + 1,
                AnnotatedSegment::source(after),
            ));
        }

        if let Some(note) = note {
            annotations.push(AnnotatedLine::annotation(vec![]));
            annotations.push(AnnotatedLine::annotation(
                AnnotatedSegment::highlight(format!(" NOTE: ")).with(AnnotatedSegment::text(note)),
            ));
        }

        let (smaller_idx, larger_idx) = if self.cause.line_idx > self.effect.line_idx {
            (self.effect.line_idx, self.cause.line_idx)
        } else {
            (self.cause.line_idx, self.effect.line_idx)
        };

        Some(Annotation::multi_line(annotations, smaller_idx, larger_idx))
    }
}

fn extract_span<'a>(line: &'a str, start: usize, end: usize) -> (String, String, String) {
    let before = line.chars().take(start).collect();
    let span = line.chars().skip(start).take(end - start).collect();
    let after = line.chars().skip(end).collect();
    (before, span, after)
}

fn find_line_and_context<'a>(
    source: &'a str,
    line_idx: usize,
) -> Option<(Option<&'a str>, &'a str, Option<&'a str>)> {
    let mut lines = source.lines();
    let before = if line_idx == 0 {
        None
    } else {
        Some(lines.nth(line_idx - 1)?)
    };
    let line = lines.next()?;
    let after = lines.next();
    Some((
        before.and_then(remove_blank),
        line,
        after.and_then(remove_blank),
    ))
}

fn remove_blank<'a>(val: &'a str) -> Option<&'a str> {
    if val.is_empty() || val.chars().all(|ch| ch.is_whitespace()) {
        None
    } else {
        Some(val)
    }
}

trait WithSegment {
    fn with(self, annotation: AnnotatedSegment) -> Vec<AnnotatedSegment>;
}

impl WithSegment for AnnotatedSegment {
    fn with(self, annotation: AnnotatedSegment) -> Vec<AnnotatedSegment> {
        vec![self, annotation]
    }
}

impl WithSegment for Vec<AnnotatedSegment> {
    fn with(mut self, annotation: AnnotatedSegment) -> Vec<AnnotatedSegment> {
        self.push(annotation);
        self
    }
}

impl From<AnnotatedSegment> for Vec<AnnotatedSegment> {
    fn from(value: AnnotatedSegment) -> Self {
        vec![value]
    }
}
