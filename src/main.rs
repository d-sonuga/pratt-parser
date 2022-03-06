use std::{env, process, fs, str, vec, fmt, iter};
use std::collections::HashMap;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Invalid number of arguments");
        process::exit(1);
    }
    let filename = args[1].clone();
    let prog = fs::read_to_string(filename).unwrap();
    let mut tokens: Vec<Token> = vec![];
    tokenize(&mut tokens, prog.chars(), false, String::new(), false, String::new());
    let mut parser = Parser::new(tokens);
    println!("{:?}", parser.parse());
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Name(String),
    Plus,
    Minus,
    Slash,
    Star,
    Tilde,
    Bang,
    Caret,
    OpenParen,
    CloseParen,
    IntLit(i32)
}

impl Token {
    fn get_type(&self) -> &'static str {
        match self {
            Token::Name(_) => "name",
            Token::Plus => "plus",
            Token::Minus => "minus",
            Token::Slash => "slash",
            Token::Star => "star",
            Token::Tilde => "tilde",
            Token::Bang => "bang",
            Token::OpenParen => "openparen",
            Token::CloseParen => "closeparen",
            Token::IntLit(_) => "intlit",
            Token::Caret => "caret"
        }
    }
    fn get_repr(&self) -> String {
        match self {
            Token::Name(name) => (*name).clone(),
            Token::Plus => format!("+"),
            Token::Minus => format!("-"),
            Token::Slash => format!("/"),
            Token::Star => format!("*"),
            Token::Tilde => format!("~"),
            Token::Bang => format!("!"),
            Token::OpenParen => format!("("),
            Token::CloseParen => format!(")"),
            Token::IntLit(lit) => format!("{}", lit),
            Token::Caret => format!("^")
        }
    }
    fn is_binop(&self) -> bool {
        match self {
            Token::Plus | Token::Minus | Token::Slash | Token::Star | Token::Caret => true,
            _ => false
        }
    }
    fn get_binding_power(&self) -> i32 {
        match self {
            Token::Plus | Token::Minus => SUM_BP,
            Token::Slash | Token::Star => PRODUCT_BP,
            Token::Caret => CARET_BP,
            _ => 0
        }
    }
}

fn tokenize(
    mut tokens: &mut Vec<Token>,
    mut prog: str::Chars,
    mut is_int: bool,
    mut curr_int: String,
    mut is_name: bool,
    mut curr_name: String
) {
    let mut summarize_int = |tokens: &mut &mut Vec<Token>| {
        if !is_int {
            return;
        }
        is_int = false;
        tokens.push(Token::IntLit(curr_int.parse::<i32>().unwrap()));
        curr_int = String::new();
    };
    let mut summarize_name = |tokens: &mut &mut Vec<Token>| {
        if !is_name {
            return;
        }
        is_name = false;
        tokens.push(Token::Name(curr_name.clone()));
        curr_name = String::new();
    };
    let c = prog.next();
    if c.is_none() {
        summarize_name(&mut tokens);
        summarize_int(&mut tokens);
        return;
    }
    let c = c.unwrap();
    match c {
        '\n' | ' '  => {
            summarize_int(&mut tokens);
            summarize_name(&mut tokens);
        },
        '+' => {
            summarize_int(&mut tokens);
            summarize_name(&mut tokens);
            tokens.push(Token::Plus);
        },
        '-' => {
            summarize_int(&mut tokens);
            summarize_name(&mut tokens);
            tokens.push(Token::Minus);
        },
        '/' => {
            summarize_int(&mut tokens);
            summarize_name(&mut tokens);
            tokens.push(Token::Slash);
        },
        '*' => {
            summarize_int(&mut tokens);
            summarize_name(&mut tokens);
            tokens.push(Token::Star);
        },
        '!' => {
            summarize_int(&mut tokens);
            summarize_name(&mut tokens);
            tokens.push(Token::Bang);
        },
        '~' => {
            summarize_int(&mut tokens);
            summarize_name(&mut tokens);
            tokens.push(Token::Tilde);
        },
        '(' => {
            summarize_int(&mut tokens);
            summarize_name(&mut tokens);
            tokens.push(Token::OpenParen);
        },
        ')' => {
            summarize_int(&mut tokens);
            summarize_name(&mut tokens);
            tokens.push(Token::CloseParen);
        },
        '^' => {
            summarize_int(&mut tokens);
            summarize_name(&mut tokens);
            tokens.push(Token::Caret);
        }
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
            summarize_name(&mut tokens);
            is_int = true;
            curr_int.push(c);
        },
        _ => {
            summarize_int(&mut tokens);
            is_name = true;
            curr_name.push(c);
        }
    }
    tokenize(tokens, prog, is_int, curr_int, is_name, curr_name);
}

struct Parser {
    tokens: iter::Peekable<vec::IntoIter<Token>>,
    prefix_parselets: HashMap<&'static str, Box<dyn PrefixParselet>>,
    infix_parselets: HashMap<&'static str, Box<dyn InfixParselet>>
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        let mut parser = Parser {
            tokens: tokens.into_iter().peekable(),
            prefix_parselets: HashMap::new(),
            infix_parselets: HashMap::new()
        };
        parser.reg_prefix_parselet("name", Box::new(NameParselet::new(PREFIX_BP)));
        parser.reg_prefix_parselet("intlit", Box::new(IntLitParselet::new(PREFIX_BP)));
        parser.reg_prefix_parselet("bang", Box::new(PrefixOpParselet::new(PREFIX_BP)));
        parser.reg_prefix_parselet("tilde", Box::new(PrefixOpParselet::new(PREFIX_BP)));
        parser.reg_prefix_parselet("minus", Box::new(PrefixOpParselet::new(PREFIX_BP)));
        parser.reg_prefix_parselet("plus", Box::new(PrefixOpParselet::new(PREFIX_BP)));

        parser.reg_infix_parselet("plus", Box::new(InfixOpParselet::new(SUM_BP)));
        parser.reg_infix_parselet("minus", Box::new(InfixOpParselet::new(SUM_BP)));
        parser.reg_infix_parselet("slash", Box::new(InfixOpParselet::new(PRODUCT_BP)));
        parser.reg_infix_parselet("star", Box::new(InfixOpParselet::new(PRODUCT_BP)));
        parser.reg_infix_parselet("caret", Box::new(InfixOpParselet::new(CARET_BP)));

        parser
    }
    fn reg_prefix_parselet(&mut self, token_type: &'static str, parselet: Box<dyn PrefixParselet>){
        self.prefix_parselets.insert(token_type, parselet);
    }
    fn reg_infix_parselet(&mut self, token_type: &'static str, parselet: Box<dyn InfixParselet>){
        self.infix_parselets.insert(token_type, parselet);
    }
    fn parse(&mut self) -> Box<dyn Expression> {
        self.parse_expression(0)
    }
    fn parse_expression(&mut self, binding_power: i32) -> Box<dyn Expression> {
        let first_token = self.tokens.next().unwrap();
        let mut left_expr;
        if first_token == Token::OpenParen {
            left_expr = self.parse_expression(0);
        } else {
            let prefix_parselet = self.prefix_parselets.get_mut(first_token.get_type()).unwrap().clone();
            left_expr = prefix_parselet.parse(self, first_token);
        }
        loop {
            let next_token = self.tokens.peek();
            if next_token.is_none() {
                break;
            }
            let next_token = next_token.unwrap();
            if *next_token == Token::CloseParen {
                self.tokens.next();
                break;
            }
            if next_token.is_binop() && next_token.get_binding_power() > binding_power {
                let token = self.tokens.next().unwrap();
                let infix_parselet = self.infix_parselets.get_mut(token.get_type());
                if infix_parselet.is_none() {
                    println!("Invalid infix operator");
                    process::exit(1);
                }
                let infix_parselet = infix_parselet.unwrap().clone();
                left_expr = infix_parselet.parse(self, left_expr, token.clone());
            } else {
                break;
            }
        }
        left_expr
    }
}

trait PrefixParselet: PrefixParseletClone {
    fn parse(&self, parser: &mut Parser, token: Token) -> Box<dyn Expression>;
}

trait PrefixParseletClone {
    fn clone_parselet(&self) -> Box<dyn PrefixParselet>;
}

impl<T> PrefixParseletClone for T
where T: 'static + PrefixParselet + Clone {
    fn clone_parselet(&self) -> Box<dyn PrefixParselet> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn PrefixParselet> {
    fn clone(&self) -> Box<dyn PrefixParselet> {
        self.clone_parselet()
    }
}

#[derive(Clone)]
struct NameParselet {
    binding_power: i32
}

impl PrefixParselet for NameParselet {
    fn parse(&self, _parser: &mut Parser, token: Token) -> Box<dyn Expression> {
        Box::new(NameExpression::new(token))
    }
}

impl NameParselet {
    fn new(binding_power: i32) -> NameParselet {
        NameParselet {
            binding_power
        }
    }
}

#[derive(Clone)]
struct PrefixOpParselet {
    binding_power: i32
}

impl PrefixParselet for PrefixOpParselet {
    fn parse(&self, parser: &mut Parser, token: Token) -> Box<dyn Expression> {
        let operand = parser.parse_expression(self.binding_power);
        Box::new(PrefixExpression::new(token, operand))
    }
}

impl PrefixOpParselet {
    fn new(binding_power: i32) -> PrefixOpParselet {
        PrefixOpParselet {
            binding_power
        }
    }
}

#[derive(Clone)]
struct IntLitParselet {
    binding_power: i32
}

impl PrefixParselet for IntLitParselet {
    fn parse(&self, _parser: &mut Parser, token: Token) -> Box<dyn Expression> {
        Box::new(IntLitExpression::new(token))
    }
}

impl IntLitParselet {
    fn new(binding_power: i32) -> IntLitParselet {
        IntLitParselet {
            binding_power
        }
    }
}

trait InfixParselet: InfixParseletClone {
    fn parse(&self, parser: &mut Parser, left: Box<dyn Expression>, token: Token) -> Box<dyn Expression>;
}

trait InfixParseletClone {
    fn clone_parselet(&self) -> Box<dyn InfixParselet>;
}

impl<T> InfixParseletClone for T
where T: 'static + Clone + InfixParselet {
    fn clone_parselet(&self) -> Box<dyn InfixParselet> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn InfixParselet> {
    fn clone(&self) -> Box<dyn InfixParselet> {
        self.clone_parselet()
    }
}

#[derive(Clone)]
struct InfixOpParselet {
    binding_power: i32
}

impl InfixParselet for InfixOpParselet {
    fn parse(&self, parser: &mut Parser, left: Box<dyn Expression>, token: Token) -> Box<dyn Expression> {
        let right = parser.parse_expression(self.binding_power);
        Box::new(BinaryExpression::new(left, right, token))
    }
}

impl InfixOpParselet {
    fn new(binding_power: i32) -> InfixOpParselet {
        InfixOpParselet {
            binding_power
        }
    }
}

trait Expression: fmt::Debug {}

struct NameExpression {
    name: String
}

impl Expression for NameExpression {}

impl NameExpression {
    fn new(token: Token) -> NameExpression {
        if let Token::Name(name) = token {
            return NameExpression {
                name
            };
        } else {
            println!("{:?} is not a valid name expression", token);
            process::exit(1);
        }
    }
}

impl fmt::Debug for NameExpression {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        print!("(name: {:?})", self.name);
        Ok(())
    }
}

struct PrefixExpression {
    operand: Box<dyn Expression>,
    operator: String
}

impl Expression for PrefixExpression {}

impl PrefixExpression {
    fn new(token: Token, operand: Box<dyn Expression>) -> PrefixExpression {
        PrefixExpression {
            operator: token.get_repr(),
            operand
        }
    }
}

impl fmt::Debug for PrefixExpression {
    fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        print!("(operator: {:?}, operand: {:?})", self.operator, self.operand);
        Ok(())
    }
}

struct BinaryExpression {
    left: Box<dyn Expression>,
    right: Box<dyn Expression>,
    operator: Token
}

impl Expression for BinaryExpression {}

impl BinaryExpression {
    fn new(left: Box<dyn Expression>, right: Box<dyn Expression>, operator: Token) -> BinaryExpression {
        BinaryExpression {
            left,
            right,
            operator
        }
    }
}

impl fmt::Debug for BinaryExpression {
    fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        print!(
            "(left operand: ({:?}), operator: {:?}, right operand: ({:?}))", 
            self.left, self.operator, self.right
        );
        Ok(())
    }
}

struct IntLitExpression {
    lit: i32
}

impl Expression for IntLitExpression {}

impl IntLitExpression {
    fn new(token: Token) -> IntLitExpression {
        if let Token::IntLit(lit) = token {
            return IntLitExpression {
                lit
            };
        } else {
            println!("Invalid int literal {:?}", token);
            process::exit(1);
        }
    }
}

impl fmt::Debug for IntLitExpression {
    fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        print!("(int: {})", self.lit);
        Ok(())
    }
}

const SUM_BP: i32 = 10;
const PRODUCT_BP: i32 = 20;
const CARET_BP: i32 = 30;
const PREFIX_BP: i32 = 40;