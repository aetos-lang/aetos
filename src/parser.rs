use crate::ast::*;
use crate::lexer::{Lexer, Token};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken { expected: String, found: String },
    
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Invalid syntax: {message}")]
    InvalidSyntax { message: String },
}

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Option<Token>,
    peek_token: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next();
        let peek_token = lexer.next();
        
        Self {
            lexer,
            current_token,
            peek_token,
        }
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.take();
        self.peek_token = self.lexer.next();
    }

    fn expect_token(&mut self, expected: Token) -> ParseResult<()> {
        if let Some(ref current) = self.current_token {
            if std::mem::discriminant(current) == std::mem::discriminant(&expected) {
                self.next_token();
                return Ok(());
            }
        }
        
        Err(ParseError::UnexpectedToken {
            expected: format!("{:?}", expected),
            found: self.current_token
                .as_ref()
                .map_or("EOF".to_string(), |t| format!("{:?}", t)),
        })
    }

    fn current_token_is(&self, token: &Token) -> bool {
        if let Some(ref current) = self.current_token {
            std::mem::discriminant(current) == std::mem::discriminant(token)
        } else {
            false
        }
    }

    fn peek_token_is(&self, token: &Token) -> bool {
        if let Some(ref peek) = self.peek_token {
            std::mem::discriminant(peek) == std::mem::discriminant(token)
        } else {
            false
        }
    }

    fn expect_identifier(&mut self) -> ParseResult<String> {
        match self.current_token.take() {
            Some(Token::Identifier(name)) => {
                self.next_token();
                Ok(name)
            }
            Some(token) => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", token),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        
        while self.current_token.is_some() {
            match &self.current_token {
                Some(Token::KeywordFn) => {
                    functions.push(self.parse_function()?);
                }
                Some(Token::KeywordStruct) => {
                    structs.push(self.parse_struct()?);
                    
                    if self.current_token_is(&Token::Semicolon) {
                        self.next_token();
                    }
                }
                Some(Token::Semicolon) => {
                    self.next_token();
                }
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected function or struct declaration".to_string(),
                    });
                }
            }
        }
        
        Ok(Program { functions, structs })
    }

    fn parse_struct(&mut self) -> ParseResult<Struct> {
        self.expect_token(Token::KeywordStruct)?;
        
        let name = self.expect_identifier()?;
        
        self.expect_token(Token::BraceOpen)?;
        
        let mut fields = Vec::new();
        while !self.current_token_is(&Token::BraceClose) {
            let field_name = self.expect_identifier()?;
            
            self.expect_token(Token::Colon)?;
            let field_type = self.parse_type()?;
            
            fields.push(StructField {
                name: field_name,
                field_type,
            });
            
            if self.current_token_is(&Token::Comma) {
                self.next_token();
            } else {
                break;
            }
        }
        
        self.expect_token(Token::BraceClose)?;
        
        Ok(Struct { name, fields })
    }

    fn parse_function(&mut self) -> ParseResult<Function> {
        self.expect_token(Token::KeywordFn)?;
        
        let name = self.expect_identifier()?;
        
        self.expect_token(Token::ParenOpen)?;
        let params = self.parse_parameters()?;
        self.expect_token(Token::ParenClose)?;
        
        self.expect_token(Token::Arrow)?;
        let return_type = self.parse_type()?;
        
        self.expect_token(Token::BraceOpen)?;
        let body = self.parse_block()?;
        self.expect_token(Token::BraceClose)?;
        
        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_parameters(&mut self) -> ParseResult<Vec<Parameter>> {
        let mut params = Vec::new();
        
        while !self.current_token_is(&Token::ParenClose) {
            let name = self.expect_identifier()?;
            
            self.expect_token(Token::Colon)?;
            let param_type = self.parse_type()?;
            
            params.push(Parameter { name, param_type });
            
            if self.current_token_is(&Token::Comma) {
                self.next_token();
            } else {
                break;
            }
        }
        
        Ok(params)
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        let token_type = match self.current_token {
            Some(Token::KeywordI32) => Type::I32,
            Some(Token::KeywordI64) => Type::I64,
            Some(Token::KeywordF32) => Type::F32,
            Some(Token::KeywordF64) => Type::F64,
            Some(Token::KeywordBool) => Type::Bool,
            Some(Token::KeywordString) => Type::String,
            Some(Token::KeywordVoid) => Type::Void,
            Some(Token::Identifier(ref name)) => Type::Struct(name.clone()),
            _ => return Err(ParseError::UnexpectedToken {
                expected: "type".to_string(),
                found: self.current_token
                    .as_ref()
                    .map_or("EOF".to_string(), |t| format!("{:?}", t)),
            }),
        };
        self.next_token();
        
        Ok(token_type)
    }

    fn parse_block(&mut self) -> ParseResult<Vec<Statement>> {
        let mut statements = Vec::new();
        
        while !self.current_token_is(&Token::BraceClose) {
            statements.push(self.parse_statement()?);
        }
        
        Ok(statements)
    }

    fn parse_statement(&mut self) -> ParseResult<Statement> {
        // Проверяем, является ли это присваиванием (идентификатор, за которым следует =)
        if let Some(Token::Identifier(_)) = &self.current_token {
            if let Some(Token::OperatorAssign) = &self.peek_token {
                // Это присваивание: name = expression;
                let name = self.expect_identifier()?;
                self.expect_token(Token::OperatorAssign)?;
                let value = self.parse_expression()?;
                self.expect_token(Token::Semicolon)?;
                return Ok(Statement::Assignment {
                    name,
                    value,
                });
            }
        }
        
        // Если не присваивание, парсим другие типы statements
        match &self.current_token {
            Some(Token::KeywordLet) => self.parse_variable_declaration(),
            Some(Token::KeywordReturn) => self.parse_return_statement(),
            Some(Token::KeywordIf) => self.parse_if_statement(),
            Some(Token::KeywordWhile) => self.parse_while_statement(),
            Some(Token::BraceOpen) => self.parse_block_statement(),
            
            // Для всех остальных случаев - это выражение
            _ => {
                let expr = self.parse_expression()?;
                self.expect_token(Token::Semicolon)?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_variable_declaration(&mut self) -> ParseResult<Statement> {
        self.expect_token(Token::KeywordLet)?;
    
        let mutable = if self.current_token_is(&Token::KeywordMut) {
            self.next_token();
            true
        } else {
            false
        };
    
        let name = self.expect_identifier()?;
    
        self.expect_token(Token::Colon)?;
        let var_type = self.parse_type()?;
    
        self.expect_token(Token::OperatorAssign)?;
        let value = self.parse_expression()?;
    
        self.expect_token(Token::Semicolon)?;
    
        Ok(Statement::VariableDeclaration {
            name,
            var_type,
            value,
            mutable,
        })
    }

    fn parse_return_statement(&mut self) -> ParseResult<Statement> {
        self.expect_token(Token::KeywordReturn)?;
        
        let value = if self.current_token_is(&Token::Semicolon) {
            Expression::IntegerLiteral(0)
        } else {
            self.parse_expression()?
        };
        
        self.expect_token(Token::Semicolon)?;
        
        Ok(Statement::Return { value })
    }

    fn parse_if_statement(&mut self) -> ParseResult<Statement> {
        self.expect_token(Token::KeywordIf)?;
        
        let condition = self.parse_expression()?;
        
        // Обрабатываем тело if (может быть блоком или одиночным statement)
        let then_branch = if self.current_token_is(&Token::BraceOpen) {
            self.parse_block()?
        } else {
            // Одиночный statement без фигурных скобок
            vec![self.parse_statement()?]
        };
        
        let else_branch = if self.current_token_is(&Token::KeywordElse) {
            self.next_token();
            
            if self.current_token_is(&Token::KeywordIf) {
                let else_if_stmt = self.parse_if_statement()?;
                Some(vec![else_if_stmt])
            } else if self.current_token_is(&Token::BraceOpen) {
                Some(self.parse_block()?)
            } else {
                // Одиночный statement без фигурных скобок
                Some(vec![self.parse_statement()?])
            }
        } else {
            None
        };
        
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> ParseResult<Statement> {
        self.expect_token(Token::KeywordWhile)?;
        
        let condition = self.parse_expression()?;
        let body = if self.current_token_is(&Token::BraceOpen) {
            self.parse_block()?
        } else {
            vec![self.parse_statement()?]
        };
        
        Ok(Statement::While { condition, body })
    }

    fn parse_block_statement(&mut self) -> ParseResult<Statement> {
        self.expect_token(Token::BraceOpen)?;
        let statements = self.parse_block()?;
        self.expect_token(Token::BraceClose)?;
        Ok(Statement::Block { statements })
    }

    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> ParseResult<Expression> {
        let expr = self.parse_logical_or()?;
        
        if self.current_token_is(&Token::OperatorAssign) {
            self.next_token();
            let value = self.parse_assignment()?;
            
            if let Expression::Variable(name) = expr {
                return Ok(Expression::BinaryExpression {
                    left: Box::new(Expression::Variable(name)),
                    operator: BinaryOperator::Eq,
                    right: Box::new(value),
                });
            } else {
                return Err(ParseError::InvalidSyntax {
                    message: "Left side of assignment must be a variable".to_string(),
                });
            }
        }
        
        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_logical_and()?;
        
        while let Some(Token::OperatorOr) = &self.current_token {
            self.next_token();
            let right = self.parse_logical_and()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_equality()?;
        
        while let Some(Token::OperatorAnd) = &self.current_token {
            self.next_token();
            let right = self.parse_equality()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_equality(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_comparison()?;
        
        while let Some(token) = &self.current_token {
            match token {
                Token::OperatorEq | Token::OperatorNeq => {
                    let operator = match token {
                        Token::OperatorEq => BinaryOperator::Eq,
                        Token::OperatorNeq => BinaryOperator::Neq,
                        _ => unreachable!(),
                    };
                    self.next_token();
                    
                    let right = self.parse_comparison()?;
                    left = Expression::BinaryExpression {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        
        Ok(left)
    }

    fn parse_comparison(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_term()?;
        
        while let Some(token) = &self.current_token {
            match token {
                Token::OperatorLt | Token::OperatorGt | Token::OperatorLte | Token::OperatorGte => {
                    let operator = match token {
                        Token::OperatorLt => BinaryOperator::Lt,
                        Token::OperatorGt => BinaryOperator::Gt,
                        Token::OperatorLte => BinaryOperator::Lte,
                        Token::OperatorGte => BinaryOperator::Gte,
                        _ => unreachable!(),
                    };
                    self.next_token();
                    
                    let right = self.parse_term()?;
                    left = Expression::BinaryExpression {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        
        Ok(left)
    }

    fn parse_term(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_factor()?;
        
        while let Some(token) = &self.current_token {
            match token {
                Token::OperatorAdd | Token::OperatorSubtract => {
                    let operator = match token {
                        Token::OperatorAdd => BinaryOperator::Add,
                        Token::OperatorSubtract => BinaryOperator::Subtract,
                        _ => unreachable!(),
                    };
                    self.next_token();
                    
                    let right = self.parse_factor()?;
                    left = Expression::BinaryExpression {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        
        Ok(left)
    }

    fn parse_factor(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_unary()?;
    
        while let Some(token) = &self.current_token {
            match token {
                Token::OperatorMultiply | Token::OperatorDivide => {
                    let operator = match token {
                        Token::OperatorMultiply => BinaryOperator::Multiply,
                        Token::OperatorDivide => BinaryOperator::Divide,
                        _ => unreachable!(),
                    };
                    self.next_token();
                    
                    let right = self.parse_unary()?;
                    left = Expression::BinaryExpression {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    };
                }
                Token::KeywordAs => {
                    self.next_token();
                    let target_type = self.parse_type()?;
                    left = Expression::TypeCast {
                        expression: Box::new(left),
                        target_type,
                    };
                }
                _ => break,
            }
        }
        
        Ok(left)
    }

    fn parse_unary(&mut self) -> ParseResult<Expression> {
        match &self.current_token {
            Some(Token::OperatorSubtract) => {
                self.next_token();
                let expr = self.parse_unary()?;
                Ok(Expression::BinaryExpression {
                    left: Box::new(Expression::IntegerLiteral(0)),
                    operator: BinaryOperator::Subtract,
                    right: Box::new(expr),
                })
            }
            Some(Token::OperatorNot) => {
                self.next_token();
                let expr = self.parse_unary()?;
                Ok(Expression::BinaryExpression {
                    left: Box::new(expr),
                    operator: BinaryOperator::Eq,
                    right: Box::new(Expression::BoolLiteral(false)),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> ParseResult<Expression> {
        match self.current_token.take() {
            Some(Token::IntegerLiteral(value)) => {
                self.next_token();
                Ok(Expression::IntegerLiteral(value))
            }
            Some(Token::FloatLiteral(value)) => {
                self.next_token();
                Ok(Expression::FloatLiteral(value))
            }
            Some(Token::StringLiteral(value)) => {
                self.next_token();
                Ok(Expression::StringLiteral(value))
            }
            Some(Token::KeywordTrue) => {
                self.next_token();
                Ok(Expression::BoolLiteral(true))
            }
            Some(Token::KeywordFalse) => {
                self.next_token();
                Ok(Expression::BoolLiteral(false))
            }
            Some(Token::Identifier(name)) => {
                self.next_token();
                
                // Проверяем специальные ключевые слова
                match name.as_str() {
                    "move" => {
                        self.expect_token(Token::ParenOpen)?;
                        let expr = self.parse_expression()?;
                        self.expect_token(Token::ParenClose)?;
                        return Ok(Expression::Move {
                            expression: Box::new(expr),
                        });
                    }
                    "borrow" | "mut_borrow" => {
                        let mutable = name == "mut_borrow";
                        self.expect_token(Token::ParenOpen)?;
                        let expr = self.parse_expression()?;
                        self.expect_token(Token::ParenClose)?;
                        return Ok(Expression::Borrow {
                            expression: Box::new(expr),
                            mutable,
                        });
                    }
                    _ => {}
                }
                
                // Проверяем, что следует дальше
                if self.current_token_is(&Token::ParenOpen) {
                    // Вызов функции
                    self.expect_token(Token::ParenOpen)?;
                    let args = self.parse_arguments()?;
                    self.expect_token(Token::ParenClose)?;
                    Ok(Expression::FunctionCall { name, args })
                } else if self.current_token_is(&Token::BraceOpen) {
                    // Инициализация структуры
                    self.expect_token(Token::BraceOpen)?;
                    let mut fields = Vec::new();
                    
                    while !self.current_token_is(&Token::BraceClose) {
                        let field_name = self.expect_identifier()?;
                        
                        self.expect_token(Token::Colon)?;
                        let value = self.parse_expression()?;
                        
                        fields.push((field_name, value));
                        
                        if self.current_token_is(&Token::Comma) {
                            self.next_token();
                        } else {
                            break;
                        }
                    }
                    
                    self.expect_token(Token::BraceClose)?;
                    
                    Ok(Expression::StructInitialization {
                        struct_name: name,
                        fields,
                    })
                } else if self.current_token_is(&Token::Dot) {
                    // Доступ к полю
                    let mut expr = Expression::Variable(name);
                    
                    while self.current_token_is(&Token::Dot) {
                        self.next_token();
                        let field_name = self.expect_identifier()?;
                        expr = Expression::FieldAccess {
                            expression: Box::new(expr),
                            field_name,
                        };
                    }
                    
                    Ok(expr)
                } else {
                    // Просто переменная
                    Ok(Expression::Variable(name))
                }
            }
            Some(Token::ParenOpen) => {
                self.next_token();
                let expr = self.parse_expression()?;
                self.expect_token(Token::ParenClose)?;
                Ok(expr)
            }
            Some(token) => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: format!("{:?}", token),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn parse_arguments(&mut self) -> ParseResult<Vec<Expression>> {
        let mut args = Vec::new();
        
        // Проверяем на пустой список аргументов
        if self.current_token_is(&Token::ParenClose) {
            return Ok(args);
        }
        
        // Парсим первый аргумент
        args.push(self.parse_expression()?);
        
        // Парсим остальные аргументы, разделенные запятыми
        while self.current_token_is(&Token::Comma) {
            self.next_token();
            args.push(self.parse_expression()?);
        }
        
        Ok(args)
    }
}