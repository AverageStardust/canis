use std::{
    ops::{Deref, Range},
    str::Chars,
};

pub struct Parser<'a> {
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    fn sample(&self, span: RichSpan) -> Sample<'a> {
        Sample(&self.source[span.as_range()], span)
    }

    fn parse_line(
        &mut self,
        cursor: &mut Cursor<'a>,
        byte: usize,
    ) -> Option<Result<Line<'a>, ParseError>> {
        if cursor.is_eof() {
            return None;
        }
        // Find the first ident (either label or instr), or comment
        let sample = match cursor.parse_ident() {
            Ok(Commentable::Content(span)) => self.sample(span),
            Ok(Commentable::Comment) => {
                // Once a comment is encountered, eat to newline
                cursor.eat_to_newline();
                println!("Found comment");
                return Some(Ok(Line::comment()));
            }

            Err(err) => return Some(Err(err)),
        };

        cursor.eat_whitespace();

        let (label, instr_sample) = if cursor.first() == ':' {
            let label = Some(Label(sample, byte.clone()));
            cursor.eat_whitespace();

            if cursor.first() == '\n' {
                cursor.eat_to_newline();
                return Some(Ok(Line {
                    label,
                    instruction: None,
                }));
            }

            let sample = match cursor.parse_ident() {
                Ok(Commentable::Content(span)) => self.sample(span),
                Ok(Commentable::Comment) => {
                    // Once a comment is encountered, eat to newline
                    cursor.eat_to_newline();
                    println!("Found comment");
                    return Some(Ok(Line {
                        label,
                        instruction: None,
                    }));
                }

                Err(err) => return Some(Err(err)),
            };

            (label, sample)
        } else {
            (None, sample)
        };

        let instruction = match self.parse_instruction(instr_sample, cursor) {
            Ok(instr) => instr,
            Err(err) => return Some(Err(err)),
        };

        match cursor.first() {
            '#' | '\n' => cursor.eat_to_newline(),
            _ => {
                return Some(Err(ParseError::ExtraContent(cursor.eat_continuous_span())));
            }
        }

        // Once a comment is encountered, eat to newline
        cursor.eat_to_newline();

        Some(Ok(Line {
            label,
            instruction: Some(instruction),
        }))
    }

    fn parse_instruction(
        &mut self,
        instr_sample: Sample<'_>,
        cursor: &mut Cursor,
    ) -> Result<InstructionLike, ParseError> {
        let op = match &*instr_sample {
            "add" => OpType::Op0000 { fun3: 0b000 },
            _ => return self.parse_pseudo(instr_sample, cursor),
        };

        let instr = match op {
            OpType::Op0000 { fun3 } => {
                let rd = cursor.parse_register_span().disallow_comment(cursor)?;
                let rs1 = cursor.parse_register_span().disallow_comment(cursor)?;
                let rs2 = cursor.parse_register_span().disallow_comment(cursor)?;

                let rd = parse_register(self.sample(rd))?;
                let rs1 = parse_register(self.sample(rs1))?;
                let rs2 = parse_register(self.sample(rs2))?;

                Instruction::r_type(rs1, fun3, rs2, rd, 0b0000)
            }
        };

        Ok(instr.into())
    }

    fn parse_pseudo(
        &mut self,
        instr_sample: Sample<'_>,
        cursor: &mut Cursor,
    ) -> Result<InstructionLike, ParseError> {
        match &*instr_sample {
            _ => return Err(ParseError::UnknownInstruction(instr_sample.span())),
        }
    }
}

struct Sample<'a>(&'a str, RichSpan);

impl Sample<'_> {
    fn span(&self) -> RichSpan {
        self.1
    }
}

impl<'a> Deref for Sample<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

enum OpType {
    Op0000 { fun3: u16 },
}

fn parse_register(register_sample: Sample<'_>) -> Result<u16, ParseError> {
    match &*register_sample {
        "ra" | "x0" => Ok(0),
        "sp" | "x1" => Ok(1),
        "t0" | "x2" => Ok(2),
        "t1" | "x3" => Ok(3),
        "s0" | "x4" => Ok(4),
        "s1" | "x5" => Ok(5),
        "a0" | "x6" => Ok(6),
        "a1" | "x7" => Ok(7),
        _ => Err(ParseError::InvalidRegister(register_sample.span())),
    }
}

fn parse_immediate(immediate_str: &str, span: RichSpan) -> Result<i16, ParseError> {
    Ok(0)
}

fn parse_immediate_unsigned(immediate_str: &str, span: RichSpan) -> Result<u16, ParseError> {
    Ok(0)
}

enum InstructionLike {
    Single(Instruction),
    PseudoExpanded(Vec<Instruction>),
}

impl InstructionLike {
    fn get_size(&self) -> usize {
        match self {
            Self::Single(instr) => instr.get_size(),
            Self::PseudoExpanded(instrs) => instrs.iter().map(|instr| instr.get_size()).sum(),
        }
    }
}

pub enum Instruction {
    Small(u16),
    Large(u32),
}

impl Instruction {
    fn get_size(&self) -> usize {
        match self {
            Self::Small(_) => 1,
            Self::Large(_) => 2,
        }
    }

    fn r_type(rs1: u16, fun3: u16, rs2: u16, rd: u16, op: u16) -> Self {
        let mut instr = 0u16;
        instr |= op & 0b0000000000001111;
        instr |= (rd << 4) & 0b0000000001110000;
        instr |= (rs2 << 7) & 0b0000001110000000;
        instr |= (fun3 << 10) & 0b0001110000000000;
        instr |= (rs1 << 13) & 0b1110000000000000;
        Self::Small(instr)
    }
}

impl From<Instruction> for InstructionLike {
    fn from(value: Instruction) -> Self {
        InstructionLike::Single(value)
    }
}

struct Label<'a>(Sample<'a>, usize);

struct Line<'a> {
    label: Option<Label<'a>>,
    instruction: Option<InstructionLike>,
}

impl<'a> Line<'a> {
    fn comment() -> Self {
        Self {
            label: None,
            instruction: None,
        }
    }
}

#[derive(Clone, Copy)]
struct Location {
    column: usize,
    char_count: usize,
}

enum Commentable<T> {
    Content(T),
    Comment,
}

impl<T> Commentable<T> {
    fn as_err(self, line: usize, location: Location) -> Result<T, ParseError> {
        match self {
            Self::Content(content) => Ok(content),
            Self::Comment => Err(ParseError::UnexpectedComment { line, location }),
        }
    }
}

trait DisallowComment {
    type Result;
    fn disallow_comment(self, cursor: &mut Cursor) -> Self::Result;
}

impl<T> DisallowComment for Result<Commentable<T>, ParseError> {
    type Result = Result<T, ParseError>;
    fn disallow_comment(self, cursor: &mut Cursor) -> Self::Result {
        self.and_then(|c| c.as_err(cursor.line, cursor.get_location()))
    }
}

const EOF_CHAR: char = '\0';

#[derive(Clone)]
struct Cursor<'a> {
    source: &'a str,
    chars: Chars<'a>,
    line: usize,
    char_count: usize,
    column: usize,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            source: input,
            chars: input.chars(),
            char_count: 0,
            column: 0,
            line: 0,
        }
    }

    fn get_location(&self) -> Location {
        Location {
            column: self.column,
            char_count: self.char_count,
        }
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next();

        if c.is_some() {
            self.char_count += 1;
            self.column += 1;
        }

        c
    }

    fn first(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    fn second(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    fn third(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    fn parse_ident(&mut self) -> Result<Commentable<RichSpan>, ParseError> {
        self.eat_whitespace();
        match self.first() {
            '#' => Ok(Commentable::Comment),

            c if c.is_ascii_alphabetic() => {
                let start = self.get_location();
                self.eat_while(|c| c.is_ascii_alphanumeric());
                let end = self.get_location();
                Ok(Commentable::Content(RichSpan::new(self.line, start, end)))
            }

            _ => Err(ParseError::InvalidIdent(self.eat_continuous_span())),
        }
    }

    fn parse_register_span(&mut self) -> Result<Commentable<RichSpan>, ParseError> {
        self.eat_whitespace();

        if self.first() == '#' {
            return Ok(Commentable::Comment);
        }

        if self.third().is_whitespace() && !self.second().is_whitespace() {
            let start = self.get_location();
            self.bump();
            self.bump();
            let end = self.get_location();
            Ok(Commentable::Content(RichSpan::new(self.line, start, end)))
        } else {
            Err(ParseError::InvalidRegister(self.eat_continuous_span()))
        }
    }

    fn parse_immediate_span(&mut self) -> Result<Commentable<RichSpan>, ParseError> {
        self.eat_whitespace();
        if self.first() == '#' {
            return Ok(Commentable::Comment);
        }

        let start = self.get_location();
        let radix = match (self.first(), self.second()) {
            ('0', 'b') => 2,
            ('0', 'o') => 8,
            ('0', 'x') => 16,
            _ => 10,
        };
        if radix != 10 {
            self.bump();
            self.bump();
        }

        if radix != 16 {
            self.eat_while(|c| c.is_ascii_digit());
        } else {
            self.eat_while(|c| c.is_ascii_hexdigit());
        }

        let first = self.first();
        if first.is_whitespace() || first == '#' {
            let end = self.get_location();
            Ok(Commentable::Content(RichSpan::new(self.line, start, end)))
        } else {
            self.eat_continuous_span();
            let end = self.get_location();
            Err(ParseError::InvalidImmediateChars(RichSpan::new(
                self.line, start, end,
            )))
        }
    }

    fn eat_continuous_span(&mut self) -> RichSpan {
        let start = self.get_location();
        self.eat_while(|c| c != '#' && c != '\n' && !c.is_whitespace());
        let end = self.get_location();

        RichSpan::new(self.line, start, end)
    }

    fn eat_while<P>(&mut self, predicate: P)
    where
        P: Fn(char) -> bool,
    {
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }

    fn eat_whitespace(&mut self) {
        self.eat_while(|c| c.is_whitespace() && c != '\n');
    }

    fn eat_to_newline(&mut self) {
        self.eat_while(|c| c != '\n');
        if !self.is_eof() {
            self.bump();
            self.column = 0;
            self.line += 1;
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct RichSpan {
    line: usize,
    start: Location,
    end: Location,
}

impl RichSpan {
    fn new(line: usize, start: Location, end: Location) -> Self {
        Self { line, start, end }
    }

    fn as_range(&self) -> Range<usize> {
        self.start.char_count..self.end.char_count
    }
}

pub(super) enum ParseError {
    // Register errors
    InvalidRegister(RichSpan),

    // Immediate errors
    InvalidImmediateChars(RichSpan),
    InvalidImmediateSize { span: RichSpan, value: u16 },
    ImmediateParseOverflow(RichSpan),

    // Instruction errors
    UnknownInstruction(RichSpan),

    // Other errors
    ExtraContent(RichSpan),
    InvalidIdent(RichSpan),
    UnexpectedComment { line: usize, location: Location },

    PassError { name: &'static str, err: String },
}
