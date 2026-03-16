use std::{
    iter::{Enumerate, Map},
    ops::Range,
};

pub trait SpannableIter<'a> {
    type Spanned: Iterator<Item = SpannedLine<'a>>;
    fn spanned(self) -> Self::Spanned;
}

impl<'a, I: Iterator<Item = &'a str>> SpannableIter<'a> for I {
    type Spanned = Map<Enumerate<I>, fn((usize, &'a str)) -> SpannedLine<'a>>;
    fn spanned(self) -> Self::Spanned {
        self.enumerate().map(span_line)
    }
}

fn span_line<'a>(indexed: (usize, &'a str)) -> SpannedLine<'a> {
    let (idx, line) = indexed;
    SpannedLine(line, Span::new(idx, 0, line.chars().count()))
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    line_idx: usize,
    start_char: usize,
    end_char: usize,
}

impl Span {
    fn new(line_idx: usize, start_char: usize, end_char: usize) -> Self {
        Self {
            line_idx,
            start_char,
            end_char,
        }
    }

    pub fn line(&self) -> usize {
        self.line_idx
    }

    pub fn start(&self) -> usize {
        self.start_char
    }

    pub fn end(&self) -> usize {
        self.end_char
    }
}

#[derive(Debug, Clone)]
pub struct SpannedLine<'a>(&'a str, Span);

impl<'a> SpannedLine<'a> {
    pub fn as_str(&self) -> &'a str {
        self.0
    }

    pub fn as_span(&self) -> Span {
        self.1
    }

    pub fn take(&self, chars: usize) -> Self {
        let size = self.0.chars().take(chars).map(|ch| ch.len_utf8()).sum();
        Self(
            &self.0[..size],
            Span::new(
                self.1.line(),
                self.1.start(),
                (self.1.start() + chars).min(self.1.end()),
            ),
        )
    }

    pub fn skip(&self, chars: usize) -> Self {
        let size = self.0.chars().take(chars).map(|ch| ch.len_utf8()).sum();
        Self(
            &self.0[size..],
            Span::new(
                self.1.line(),
                (self.1.start() + chars).min(self.1.end()),
                self.1.end(),
            ),
        )
    }

    pub fn split_once(&self, delimiter: char) -> Option<(Self, Self)> {
        let (left_str, right_str) = self.0.split_once(delimiter)?;
        let left = self.1.start_char;
        let right = self.1.end_char;
        let center = left + left_str.chars().count();
        let line_idx = self.1.line_idx;
        let left_span = Span::new(line_idx, left, center);
        let right_span = Span::new(line_idx, center + 1, right);
        Some((Self(left_str, left_span), Self(right_str, right_span)))
    }

    pub fn trim(&self) -> Self {
        let mut iter = self.0.char_indices().enumerate();
        if let Some((left_idx, (left_byte, left_char))) =
            iter.find(|(_, (_, c))| !c.is_whitespace())
        {
            let mut last_non_ws_idx = left_idx;
            let mut last_non_ws_byte = left_byte;
            let mut last_non_ws_char = left_char;

            while let Some((idx, (by, ch))) = iter.next() {
                if !ch.is_whitespace() {
                    last_non_ws_idx = idx;
                    last_non_ws_byte = by;
                    last_non_ws_char = ch;
                }
            }

            let right_idx = last_non_ws_idx + 1;
            let right_byte = last_non_ws_byte + last_non_ws_char.len_utf8();

            Self(
                &self.0[left_byte..right_byte],
                Span::new(
                    self.1.line_idx,
                    self.1.start_char + left_idx,
                    self.1.start_char + right_idx,
                ),
            )
        } else {
            let start = self.1.start_char;
            Self("", Span::new(self.1.line_idx, start, start))
        }
    }

    pub fn collapse(&self) -> Option<Self> {
        if self.is_empty() {
            None
        } else {
            let trimmed = self.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn break_and_trim(&self) -> (Self, Self) {
        // `left` is guaranteed not to have any whitespace, `right` may have whitespace and should be trimmed
        if let Some((left, right)) = self.trim().split_once(' ') {
            let right = right.trim();
            (left, right)
        } else {
            let trimmed = self.trim();
            let span = trimmed.as_span();
            (
                trimmed,
                Self(
                    "",
                    Span {
                        line_idx: span.line(),
                        start_char: span.end(),
                        end_char: span.end(),
                    },
                ),
            )
        }
    }

    #[cfg(test)]
    pub fn mock(content: &'a str) -> Self {
        Self(content, Span::new(0, 0, content.chars().count()))
    }

    #[cfg(test)]
    pub fn mock_with_span(content: &'a str, span: Span) -> Self {
        Self(content, span)
    }
}
