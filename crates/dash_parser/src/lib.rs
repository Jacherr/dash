use dash_middle::lexer::token::Token;
use dash_middle::lexer::token::TokenType;
use dash_middle::parser::error::Error;
use dash_middle::parser::error::ErrorKind;
use dash_middle::parser::statement::FuncId;
use dash_middle::parser::statement::Statement;
use dash_middle::util::Counter;
use dash_middle::util::LevelStack;
use stmt::StatementParser;

mod expr;
mod stmt;
mod types;

/// "Forces" a token to have a borrowed lexeme, or emit an error if it is not.
/// Evaluates to an option such that it can be used in combination with `?` in functions
/// returning `Option`s, which is the case almost everywhere in the parser.
///
/// This is a macro because this borrowing pattern cannot be expressed as a function,
/// as it would need to (1) mutably borrow the parser to potentially create an error and
/// (2) immutably borrow the token which automatically also immutably borrows the parser.
/// However with this macro, the compiler can see that the borrows are disjoint, i.e.
/// it only clones the token if it has to due to an error.
#[macro_export]
macro_rules! must_borrow_lexeme {
    ($parser:expr, $tok:expr) => {
        match $tok {
            dash_middle::lexer::token::Token {
                full: std::borrow::Cow::Borrowed(s),
                ..
            } => Some(*s),
            tok => {
                let tok = tok.clone();

                $parser.create_error(ErrorKind::UnexpectedToken(tok.clone(), TokenType::Identifier));
                None
            }
        }
    };
}

/// A JavaScript source code parser
pub struct Parser<'a> {
    tokens: Box<[Token<'a>]>,
    errors: Vec<Error<'a>>,
    error_sync: bool,
    idx: usize,
    input: &'a [u8],
    new_level_stack: LevelStack,
    function_counter: Counter<FuncId>,
}

impl<'a> Parser<'a> {
    #[cfg(feature = "from_string")]
    pub fn from_str(input: &'a str) -> Result<Self, Vec<dash_middle::lexer::error::Error<'a>>> {
        dash_lexer::Lexer::new(input)
            .scan_all()
            .map(|tok| Self::new(input, tok))
    }

    /// Creates a new parser from tokens generated by a [Lexer]
    pub fn new(input: &'a str, tokens: Vec<Token<'a>>) -> Self {
        let mut level_stack = LevelStack::new();
        level_stack.add_level();

        Self {
            tokens: tokens.into_boxed_slice(),
            errors: Vec::new(),
            error_sync: false,
            idx: 0,
            input: input.as_bytes(),
            new_level_stack: level_stack,
            // FuncId::ROOT (0) is reserved for the root function, so the counter for new functions has to start at 1
            function_counter: Counter::with(FuncId::FIRST_NON_ROOT),
        }
    }

    /// Attempts to parse a single statement
    /// If an error occurs, `None` is returned and an error is added to
    /// an internal errors vec
    /// Usually `parse_all` is used to attempt to parse the entire program
    /// and get any existing errors
    pub fn parse(&mut self) -> Option<Statement<'a>> {
        self.parse_statement()
    }

    /// Iteratively parses every token and returns an AST, or a vector of errors
    ///
    /// The AST will be folded by passing true as the `fold` parameter.
    pub fn parse_all(mut self) -> Result<(Vec<Statement<'a>>, Counter<FuncId>), Vec<Error<'a>>> {
        let mut stmts = Vec::new();

        while !self.is_eof() {
            if let Some(stmt) = self.parse() {
                stmts.push(stmt);
            }
        }

        if !self.errors.is_empty() {
            Err(self.errors)
        } else {
            Ok((stmts, self.function_counter))
        }
    }

    /// Parses a prefixed number literal (0x, 0o, 0b) and returns the number
    pub fn parse_prefixed_number_literal(&mut self, full: &str, radix: u32) -> Option<f64> {
        let src = &full[2..];
        match u64::from_str_radix(src, radix).map(|x| x as f64) {
            Ok(f) => Some(f),
            Err(e) => {
                self.create_error(ErrorKind::ParseIntError(self.previous().cloned()?, e));
                None
            }
        }
    }

    fn is_eof(&self) -> bool {
        self.idx >= self.tokens.len()
    }

    fn expect(&self, expected_ty: &'static [TokenType]) -> bool {
        match self.current() {
            Some(Token { ty, .. }) => expected_ty.contains(ty),
            _ => false,
        }
    }

    fn expect_previous(&mut self, ty: &'static [TokenType], emit_error: bool) -> bool {
        let current = match self.previous() {
            Some(k) => k,
            None => {
                if emit_error {
                    self.create_error(ErrorKind::UnexpectedEof);
                }
                return false;
            }
        };

        let ok = ty.iter().any(|ty| ty.eq(&current.ty));

        if !ok && emit_error {
            let current = current.clone();
            self.create_error(ErrorKind::UnexpectedTokenMultiple(current, ty));
        }

        ok
    }

    fn expect_identifier_and_skip(&mut self, emit_error: bool) -> bool {
        self.expect_and_skip(&[TokenType::Identifier, TokenType::Dollar], emit_error)
    }

    fn expect_and_skip(&mut self, ty: &'static [TokenType], emit_error: bool) -> bool {
        let current = match self.current() {
            Some(k) => k,
            None => {
                if emit_error {
                    self.create_error(ErrorKind::UnexpectedEof);
                }
                return false;
            }
        };

        let ok = ty.iter().any(|ty| ty.eq(&current.ty));

        if ok {
            self.advance();
        } else if emit_error {
            let current = current.clone();
            self.create_error(ErrorKind::UnexpectedTokenMultiple(current, ty));
        }

        ok
    }

    fn create_error(&mut self, kind: ErrorKind<'a>) {
        if !self.error_sync {
            self.errors.push(Error {
                kind,
                source: self.input,
            });
            self.error_sync = true;
        }
    }

    fn advance(&mut self) {
        self.idx += 1;
    }

    fn advance_back(&mut self) {
        self.idx -= 1;
    }

    fn current(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.idx)
    }

    fn previous(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.idx - 1)
    }

    fn next(&mut self) -> Option<&Token<'a>> {
        self.advance();
        self.previous()
    }

    pub fn next_identifier(&mut self) -> Option<&'a str> {
        let next = match self.next() {
            Some(tok) => tok,
            None => {
                self.create_error(ErrorKind::UnexpectedEof);
                return None;
            }
        };
        must_borrow_lexeme!(self, next)
    }
}
