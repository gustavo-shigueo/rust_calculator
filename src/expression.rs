use std::str::FromStr;

use color_eyre::{
    eyre::{eyre, OptionExt},
    Report,
};

#[derive(Debug)]
pub enum Expression {
    BinaryOperation(BinaryOperationExpr),
    UnaryNegation(Box<Expression>),
    Parenthesis(Box<Expression>),
    Number(f64),
}

impl Expression {
    pub fn evaluate(self) -> f64 {
        match self {
            Self::BinaryOperation(BinaryOperationExpr {
                operator,
                left,
                right,
            }) => match operator {
                BinaryOperator::Addition => left.evaluate() + right.evaluate(),
                BinaryOperator::Subtraction => left.evaluate() - right.evaluate(),
                BinaryOperator::Multiplication => left.evaluate() * right.evaluate(),
                BinaryOperator::Division => left.evaluate() / right.evaluate(),
                BinaryOperator::Exponentiation => left.evaluate().powf(right.evaluate()),
            },
            Self::UnaryNegation(expression) => -expression.evaluate(),
            Self::Parenthesis(expression) => expression.evaluate(),
            Self::Number(x) => x,
        }
    }
}

impl FromStr for Expression {
    type Err = Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_matches(&[' ', '\n', '\r', '\t']);
        let parenthesis_indices = locate_parenthesis(s);

        let min_priority_index = find_minimum_priority_token(s, &parenthesis_indices);
        let minimum_priority_token = s
            .chars()
            .nth(min_priority_index)
            .ok_or_eyre("Index out of range")?;

        Ok(match minimum_priority_token {
            '-' if min_priority_index == 0 => {
                Self::UnaryNegation(Self::from_str(&s[min_priority_index + 1..])?.into())
            }
            x @ ('+' | '-' | '*' | '/' | '^') => Self::BinaryOperation(BinaryOperationExpr {
                operator: x.try_into()?,
                left: Self::from_str(&s[..min_priority_index])?.into(),
                right: Self::from_str(&s[min_priority_index + 1..])?.into(),
            }),
            '(' => Self::Parenthesis(Self::from_str(&s[1..s.len() - 1])?.into()),
            _ => Self::Number(s.parse()?),
        })
    }
}

#[derive(Debug)]
pub struct BinaryOperationExpr {
    pub operator: BinaryOperator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug)]
pub enum BinaryOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Exponentiation,
}

impl TryFrom<char> for BinaryOperator {
    type Error = Report;

    fn try_from(s: char) -> Result<Self, Self::Error> {
        match s {
            '+' => Ok(Self::Addition),
            '-' => Ok(Self::Subtraction),
            '*' => Ok(Self::Multiplication),
            '/' => Ok(Self::Division),
            '^' => Ok(Self::Exponentiation),
            _ => Err(eyre!("Invalid operator")),
        }
    }
}

fn locate_parenthesis(s: &str) -> Vec<(usize, usize)> {
    let mut parenthesis_indices = vec![];
    let mut depth = 0;

    let mut current = usize::MAX;
    for (i, token) in s.char_indices().filter(|&(_, c)| matches!(c, '(' | ')')) {
        if token == '(' {
            if depth == 0 {
                current = i;
            }

            depth += 1;
        } else {
            if depth == 1 {
                parenthesis_indices.push((current, i));
            }

            depth -= 1;
        }
    }

    parenthesis_indices
}

fn find_minimum_priority_token(s: &str, parenthesis_indices: &[(usize, usize)]) -> usize {
    let mut index_iter = parenthesis_indices.iter();
    let mut current_index = index_iter.next();

    let mut min_priority_index = (usize::MAX, u8::MAX);
    for (i, token) in s.char_indices() {
        if let Some((start, end)) = current_index {
            if i > *start && i < *end {
                continue;
            }
        }

        let is_negation = token == '-'
            && (i == 0
                || s[..i]
                    .chars()
                    .rev()
                    .find(|c| !c.is_ascii_whitespace())
                    .is_some_and(|x| matches!(x, '+' | '-' | '*' | '/' | '^')));

        let priority = match token {
            ' ' | '\n' | '\r' => continue,
            '(' => 5,
            '-' if is_negation => 4,
            '^' => 3,
            '*' | '/' => 2,
            '+' | '-' => 1,
            _ => u8::MAX,
        };

        match token {
            ')' => current_index = index_iter.next(),
            '-' if is_negation => {
                // Negations need to be handled from left to right otherwise
                // they can't be properly parsed
                if min_priority_index.1 > priority {
                    min_priority_index = (i, priority);
                }
            }
            _ => {
                if min_priority_index.1 >= priority {
                    min_priority_index = (i, 6);
                }
            }
        }
    }

    min_priority_index.0
}
