use std::fmt;
use std::io::Read;
use std::str::from_utf8;
use std::rc::Rc;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::error::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType
{
    Fun,
    Return,
    Struct,
    Let,
    If,
    Else,
    Loop,
    While,
    Break,
    Extern,
    Ref,
    Deref,
    Of,
    New,
    Sizeof,

    Int,
    Char,
    Bool,
    Any,

    OpenBracket,
    CloseBracket,
    OpenSquiggly,
    CloseSquiggly,
    OpenSquare,
    CloseSquare,
    Colon,
    Comma,
    Arrow,

    Plus,
    Star,
    Minus,
    GreaterThan,
    LessThan,
    Dot,
    Equals,

    IntLiteral,
    StringLiteral,
    CharLiteral,
    BoolLiteral,
    Identifier,

    Error,
}

impl fmt::Display for TokenType
{

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            Self::Fun => write!(f, "fun"),
            Self::Return => write!(f, "return"),
            Self::Struct => write!(f, "struct"),
            Self::Let => write!(f, "let"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::Loop => write!(f, "loop"),
            Self::While => write!(f, "while"),
            Self::Break => write!(f, "break"),
            Self::Extern => write!(f, "extern"),
            Self::Ref => write!(f, "ref"),
            Self::Deref => write!(f, "deref"),
            Self::Of => write!(f, "of"),
            Self::New => write!(f, "new"),
            Self::Sizeof => write!(f, "sizeof"),

            Self::Int => write!(f, "int"),
            Self::Char => write!(f, "char"),
            Self::Bool => write!(f, "bool"),
            Self::Any => write!(f, "any"),

            Self::OpenBracket => write!(f, "("),
            Self::CloseBracket => write!(f, ")"),
            Self::OpenSquiggly => write!(f, "{{"),
            Self::CloseSquiggly => write!(f, "}}"),
            Self::OpenSquare => write!(f, "["),
            Self::CloseSquare => write!(f, "]"),
            Self::Colon => write!(f, ":"),
            Self::Comma => write!(f, ","),
            Self::Arrow => write!(f, "->"),

            Self::Plus => write!(f, "+"),
            Self::Star => write!(f, "*"),
            Self::Minus => write!(f, "-"),
            Self::GreaterThan => write!(f, ">"),
            Self::LessThan => write!(f, "<"),
            Self::Dot => write!(f, "."),
            Self::Equals => write!(f, "="),

            Self::IntLiteral => write!(f, "Int Literal"),
            Self::StringLiteral => write!(f, "String Literal"),
            Self::CharLiteral => write!(f, "Char Literal"),
            Self::BoolLiteral => write!(f, "Bool Literal"),
            Self::Identifier => write!(f, "Identifier"),

            Self::Error => write!(f, "Error"),
        }
    }

}

#[derive(Debug, Clone, PartialEq)]
struct TokenPosition
{
    file_path: Rc<String>,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token
{
    token_type: TokenType,
    content: String,
    position: TokenPosition,
}

impl Token
{

    fn new(token_type: TokenType, content: &str,
           position: TokenPosition) -> Self
    {
        Self
        {
            token_type,
            content: content.to_owned(),
            position,
        }
    }

    pub fn token_type(&self) -> &TokenType
    {
        &self.token_type
    }

    pub fn content(&self) -> &str
    {
        &self.content
    }

    pub fn position(&self) -> String
    {
        format!("{}:{}:{}",
            self.position.file_path,
            self.position.line,
            self.position.column)
    }

    pub fn show(&self) -> String
    {
        let file_or_err = File::open(self.position.file_path.as_ref());
        if file_or_err.is_err() {
            return format!("{}", file_or_err.unwrap_err());
        }

        let reader = BufReader::new(file_or_err.unwrap());
        let line_or_none = reader.lines().nth(self.position.line - 1);
        if line_or_none.is_none()
        {
            return format!("No line {} in file '{}'",
                self.position.line, self.position.file_path);
        }

        let line_or_err = line_or_none.unwrap();
        if line_or_err.is_err() {
            return format!("{}", line_or_err.unwrap_err());
        }

        let line = line_or_err.unwrap();
        let mut position_indicator = String::default();
        for _ in 0..self.position.column-1 {
            position_indicator += " ";
        }
        for _ in 0..self.content.len() {
            position_indicator += "^";
        }

        format!("{}\n{}", line, position_indicator)
    }

}

enum State
{
    Initial,
    Identifier,
    Number,
    String,
    Char,
    Slash,
    SingleLineComment,
    MultiLineComment,
    MultiLineCommentStar,
    DoubleToken(char, char, TokenType, TokenType),
}

enum StateTransition
{
    Keep(State),
    Consume(State),
}

struct Lexer
{
    tokens: Vec<Token>,
    buffer: Vec<u8>,
    buffer_start_position: Option<TokenPosition>,
}

impl Lexer
{

    fn emit_buffer_as_token(&mut self, token_type: TokenType)
    {
        let position = self.buffer_start_position.clone().unwrap();
        let token = match from_utf8(&self.buffer)
        {
            Ok(text) => Token::new(token_type, text, position),
            Err(err) =>
            {
                Token::new(TokenType::Error, &format!(
                    "Unable to parse identifier: {}", err), position)
            },
        };

        self.tokens.push(token);
        self.buffer.clear();
        self.buffer_start_position = None;
    }

}

fn handle_initial(byte: u8,
                  position: TokenPosition,
                  lexer: &mut Lexer) -> StateTransition
{
    lexer.buffer_start_position = Some(position.clone());
    let mut emit = |token_type: TokenType, content: &str|
    {
        lexer.tokens.push(Token::new(token_type, content, position.clone()));
        StateTransition::Consume(State::Initial)
    };

    match byte as char
    {
        'a'..='z' | 'A'..='Z' => StateTransition::Keep(State::Identifier),
        '0'..='9' => StateTransition::Keep(State::Number),
        ' ' | '\n' | '\t' => StateTransition::Consume(State::Initial),
        '(' => emit(TokenType::OpenBracket, "("),
        ')' => emit(TokenType::CloseBracket, ")"),
        '{' => emit(TokenType::OpenSquiggly, "{"),
        '}' => emit(TokenType::CloseSquiggly, "}"),
        '[' => emit(TokenType::OpenSquare, "["),
        ']' => emit(TokenType::CloseSquare, "]"),
        ':' => emit(TokenType::Colon, ":"),
        ',' => emit(TokenType::Comma, ","),
        '+' => emit(TokenType::Plus, "+"),
        '*' => emit(TokenType::Star, "*"),
        '>' => emit(TokenType::GreaterThan, ">"),
        '<' => emit(TokenType::LessThan, "<"),
        '.' => emit(TokenType::Dot, "."),
        '=' => emit(TokenType::Equals, "="),
        '-' => StateTransition::Consume(State::DoubleToken('-', '>', TokenType::Minus, TokenType::Arrow)),
        '#' => StateTransition::Consume(State::SingleLineComment),
        '/' => StateTransition::Consume(State::Slash),
        '\"' => StateTransition::Consume(State::String),
        '\'' => StateTransition::Consume(State::Char),

        _ => 
        {
            lexer.tokens.push(Token::new(TokenType::Error,&format!(
                "Unexpected token '{}'", byte as char), position));
            StateTransition::Consume(State::Initial)
        },
    }
}

fn parse_identifier(text: &str) -> TokenType
{
    if text == "fun" {
        TokenType::Fun
    } else if text == "return" {
        TokenType::Return
    } else if text == "struct" {
        TokenType::Struct
    } else if text == "let" {
        TokenType::Let
    } else if text == "if" {
        TokenType::If
    } else if text == "else" {
        TokenType::Else
    } else if text == "loop" {
        TokenType::Loop
    } else if text == "while" {
        TokenType::While
    } else if text == "break" {
        TokenType::Break
    } else if text == "extern" {
        TokenType::Extern
    } else if text == "ref" {
        TokenType::Ref
    } else if text == "deref" {
        TokenType::Deref
    } else if text == "int" {
        TokenType::Int
    } else if text == "char" {
        TokenType::Char
    } else if text == "bool" {
        TokenType::Bool
    } else if text == "any" {
        TokenType::Any
    } else if text == "of" {
        TokenType::Of
    } else if text == "new" {
        TokenType::New
    } else if text == "sizeof" {
        TokenType::Sizeof
    } else if text == "true" {
        TokenType::BoolLiteral
    } else if text == "false" {
        TokenType::BoolLiteral
    } else {
        TokenType::Identifier
    }
}

fn handle_identifier(byte: u8, lexer: &mut Lexer) -> StateTransition
{
    match byte as char
    {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' =>
        {
            lexer.buffer.push(byte);
            StateTransition::Consume(State::Identifier)
        },
        
        _ =>
        {
            let token = match from_utf8(&lexer.buffer)
            {
                Ok(text) =>
                {
                    Token::new(parse_identifier(text), text,
                        lexer.buffer_start_position.clone().unwrap())
                },

                Err(err) =>
                {
                    Token::new(TokenType::Error,
                        &format!("Unable to parse identifier: {}", err),
                        lexer.buffer_start_position.clone().unwrap())
                }
            };

            lexer.tokens.push(token);
            lexer.buffer.clear();
            StateTransition::Keep(State::Initial)
        }
    }
}

fn handle_number(byte: u8, lexer: &mut Lexer) -> StateTransition
{
    match byte as char
    {
        '0'..='9' =>
        {
            lexer.buffer.push(byte);
            StateTransition::Consume(State::Number)
        }

        _ =>
        {
            lexer.emit_buffer_as_token(TokenType::IntLiteral);
            StateTransition::Keep(State::Initial)
        },
    }
}

fn handle_string(byte: u8, lexer: &mut Lexer) -> StateTransition
{
    match byte as char
    {
        '\"' =>
        {
            lexer.emit_buffer_as_token(TokenType::StringLiteral);
            StateTransition::Consume(State::Initial)
        },

        _ =>
        {
            lexer.buffer.push(byte);
            StateTransition::Consume(State::String)
        }
    }
}

fn handle_char(byte: u8, lexer: &mut Lexer) -> StateTransition
{
    match byte as char
    {
        '\'' =>
        {
            lexer.emit_buffer_as_token(TokenType::CharLiteral);
            StateTransition::Consume(State::Initial)
        },

        _ =>
        {
            assert!(lexer.buffer.is_empty());
            lexer.buffer.push(byte);
            StateTransition::Consume(State::Char)
        },
    }
}

fn handle_slash(byte: u8) -> StateTransition
{
    if byte as char == '*' {
        StateTransition::Consume(State::MultiLineComment)
    } else {
        // FIXME: This need to be an error
        StateTransition::Consume(State::Initial)
    }
}

fn handle_double_token(byte: u8,
                       first: char, second: char,
                       single: &TokenType, double: &TokenType,
                       position: TokenPosition,
                       lexer: &mut Lexer)
    -> StateTransition
{
    if byte as char == second
    {
        lexer.tokens.push(Token::new(double.clone(), &format!(
            "{}{}", first, second), position));
        StateTransition::Consume(State::Initial)
    }
    else
    {
        lexer.tokens.push(Token::new(single.clone(), &format!(
            "{}", first), position));
        StateTransition::Keep(State::Initial)
    }
}

fn handle_single_line_comment(byte: u8) -> StateTransition
{
    if byte as char == '\n' {
        StateTransition::Consume(State::Initial)
    } else {
        StateTransition::Consume(State::SingleLineComment)
    }
}

fn handle_multi_line_comment(byte: u8) -> StateTransition
{
    if byte as char == '*' {
        StateTransition::Consume(State::MultiLineCommentStar)
    } else {
        StateTransition::Consume(State::MultiLineComment)
    }
}

fn handle_multi_line_comment_star(byte: u8) -> StateTransition
{
    if byte as char == '/' {
        StateTransition::Consume(State::Initial)
    } else {
        StateTransition::Consume(State::MultiLineComment)
    }
}

pub fn tokenize(file_path: &str, source_code: impl Read)
    -> Result<impl Iterator<Item = Token>, Box<dyn Error>>
{
    let mut state = State::Initial;
    let mut lexer = Lexer
    {
        tokens: Vec::new(),
        buffer: Vec::new(),
        buffer_start_position: None,
    };

    let mut bytes = source_code.bytes();
    let first_byte_or_none = bytes.next();
    if first_byte_or_none.is_none() {
        return Ok(lexer.tokens.into_iter());
    }

    let mut current_byte = first_byte_or_none.unwrap()?;
    let mut position = TokenPosition
    {
        file_path: Rc::from(file_path.to_owned()),
        line: if current_byte as char == '\n' { 2 } else { 1 },
        column: if current_byte as char == '\n' { 1 } else { 2 },
    };

    loop
    {
        let transition = match state
        {
            State::Initial => handle_initial(current_byte, position.clone(), &mut lexer),
            State::Identifier => handle_identifier(current_byte, &mut lexer),
            State::Number => handle_number(current_byte, &mut lexer),
            State::String => handle_string(current_byte, &mut lexer),
            State::Char => handle_char(current_byte, &mut lexer),
            State::Slash => handle_slash(current_byte),
            State::SingleLineComment => handle_single_line_comment(current_byte),
            State::MultiLineComment => handle_multi_line_comment(current_byte),
            State::MultiLineCommentStar => handle_multi_line_comment_star(current_byte),

            State::DoubleToken(first, second, ref single, ref double) =>
            {
                handle_double_token(current_byte,
                    first, second, single, double,
                    position.clone(), &mut lexer)
            },
        };

        state = match transition
        {
            StateTransition::Keep(new_state) => new_state,
            StateTransition::Consume(new_state) =>
            {
                let next_byte = bytes.next();
                if next_byte.is_none() {
                    break;
                }
                current_byte = next_byte.unwrap()?;
                position.column += 1;

                if current_byte as char == '\n'
                {
                    position.line += 1;
                    position.column = 0;
                }

                new_state
            },            
        };
    }

    Ok(lexer.tokens.into_iter())
}

