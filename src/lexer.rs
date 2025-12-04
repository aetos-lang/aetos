use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Ключевые слова
    #[token("fn")]
    KeywordFn,
    
    #[token("let")]
    KeywordLet,
    
    #[token("mut")]
    KeywordMut,

    #[token("as")]
    KeywordAs,
    
    #[token("return")]
    KeywordReturn,
    
    #[token("if")]
    KeywordIf,
    
    #[token("else")]
    KeywordElse,
    
    #[token("while")]
    KeywordWhile,

    #[token("for")]
    KeywordFor,

    #[token("in")]
    KeywordIn,
    
    #[token("struct")]
    KeywordStruct,
    
    #[token("true")]
    KeywordTrue,
    
    #[token("false")]
    KeywordFalse,
    
    #[token("i32")]
    KeywordI32,
    
    #[token("i64")]
    KeywordI64,
    
    #[token("f32")]
    KeywordF32,
    
    #[token("f64")]
    KeywordF64,
    
    #[token("bool")]
    KeywordBool,
    
    #[token("string")]
    KeywordString,
    
    #[token("void")]
    KeywordVoid,

    // Идентификаторы
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Литералы
    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    IntegerLiteral(i32),
    
    #[regex("[0-9]+\\.[0-9]+", |lex| lex.slice().parse().ok())]
    FloatLiteral(f32),
    
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLiteral(String),

    // Операторы
    #[token("+")]
    OperatorAdd,
    
    #[token("-")]
    OperatorSubtract,
    
    #[token("*")]
    OperatorMultiply,
    
    #[token("/")]
    OperatorDivide,

    #[token("%")]
    OperatorModulo,
    
    #[token("=")]
    OperatorAssign,
    
    #[token("==")]
    OperatorEq,
    
    #[token("!=")]
    OperatorNeq,
    
    #[token("!")]
    OperatorNot,
    
    #[token("<")]
    OperatorLt,
    
    #[token(">")]
    OperatorGt,
    
    #[token("<=")]
    OperatorLte,
    
    #[token(">=")]
    OperatorGte,
    
    #[token("&&")]
    OperatorAnd,
    
    #[token("||")]
    OperatorOr,

    #[token("?")]
    Question,

    #[token(":")]
    Colon,

    // Разделители
    #[token("(")]
    ParenOpen,
    
    #[token(")")]
    ParenClose,
    
    #[token("{")]
    BraceOpen,
    
    #[token("}")]
    BraceClose,
    
    #[token("[")]
    BracketOpen,
    
    #[token("]")]
    BracketClose,
    
    #[token(";")]
    Semicolon,
    
    #[token(",")]
    Comma,
    
    #[token(".")]
    Dot,
    
    #[token("->")]
    Arrow,

    // Комментарии и пробелы (игнорируются)
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: Token::lexer(input),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.inner.next().and_then(Result::ok);
        println!("DEBUG LEXER: {:?}", token);
        token
    }
}