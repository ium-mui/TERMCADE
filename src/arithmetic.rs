use std::fmt;
use std::ops::RangeInclusive;
use std::sync::Arc;

use rand::Rng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

impl Operation {
    pub fn symbol(self) -> char {
        match self {
            Self::Addition => '+',
            Self::Subtraction => '−',
            Self::Multiplication => '×',
            Self::Division => '÷',
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Addition => "덧셈",
            Self::Subtraction => "뺄셈",
            Self::Multiplication => "곱셈",
            Self::Division => "나눗셈",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArithmeticProblem {
    pub left: i64,
    pub operation: Operation,
    pub right: i64,
    pub answer: i64,
}

impl ArithmeticProblem {
    pub fn new(left: i64, operation: Operation, right: i64) -> Self {
        let answer = match operation {
            Operation::Addition => left + right,
            Operation::Subtraction => left - right,
            Operation::Multiplication => left * right,
            Operation::Division => left / right,
        };
        Self {
            left,
            operation,
            right,
            answer,
        }
    }

    pub fn equation(&self) -> String {
        format!(
            "{} {} {} = ?",
            self.left,
            self.operation.symbol(),
            self.right
        )
    }

    pub fn is_correct(&self, input: &str, format: AnswerFormat) -> bool {
        format
            .parse(input)
            .is_ok_and(|answer| answer == self.answer)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AnswerFormat {
    Integer,
    Fraction,
    QuotientRemainder,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum AnswerParseError {
    #[error("답을 입력해 주세요")]
    Empty,
    #[error("정수 답안만 입력할 수 있습니다")]
    InvalidInteger,
    #[error("분수 답안 형식은 a/b 입니다")]
    InvalidFraction,
    #[error("몫과 나머지 답안 형식은 몫:나머지 입니다")]
    InvalidQuotientRemainder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedAnswer {
    pub value: i64,
}

impl AnswerFormat {
    pub fn parse(self, input: &str) -> Result<i64, AnswerParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(AnswerParseError::Empty);
        }

        match self {
            Self::Integer => input
                .parse::<i64>()
                .map_err(|_| AnswerParseError::InvalidInteger),
            Self::Fraction => parse_fraction(input),
            Self::QuotientRemainder => parse_quotient_remainder(input),
        }
    }
}

fn parse_fraction(input: &str) -> Result<i64, AnswerParseError> {
    let (numerator, denominator) = input
        .split_once('/')
        .ok_or(AnswerParseError::InvalidFraction)?;
    let numerator = numerator
        .trim()
        .parse::<i64>()
        .map_err(|_| AnswerParseError::InvalidFraction)?;
    let denominator = denominator
        .trim()
        .parse::<i64>()
        .map_err(|_| AnswerParseError::InvalidFraction)?;
    if denominator == 0 || numerator % denominator != 0 {
        return Err(AnswerParseError::InvalidFraction);
    }
    Ok(numerator / denominator)
}

fn parse_quotient_remainder(input: &str) -> Result<i64, AnswerParseError> {
    let (quotient, remainder) = input
        .split_once(':')
        .ok_or(AnswerParseError::InvalidQuotientRemainder)?;
    let quotient = quotient
        .trim()
        .parse::<i64>()
        .map_err(|_| AnswerParseError::InvalidQuotientRemainder)?;
    let remainder = remainder
        .trim()
        .parse::<i64>()
        .map_err(|_| AnswerParseError::InvalidQuotientRemainder)?;
    if remainder < 0 {
        return Err(AnswerParseError::InvalidQuotientRemainder);
    }
    Ok(quotient + remainder)
}

pub trait ProblemGenerator: Send + Sync {
    fn generate(&self, rng: &mut StdRng) -> ArithmeticProblem;
}

#[derive(Clone, Debug)]
pub struct ArithmeticGenerator {
    operation: Operation,
    range: RangeInclusive<i64>,
}

impl ArithmeticGenerator {
    pub fn new(operation: Operation, range: RangeInclusive<i64>) -> Self {
        Self { operation, range }
    }

    pub fn operation(&self) -> Operation {
        self.operation
    }
}

impl ProblemGenerator for ArithmeticGenerator {
    fn generate(&self, rng: &mut StdRng) -> ArithmeticProblem {
        match self.operation {
            Operation::Addition => {
                let left = rng.gen_range(self.range.clone());
                let right = rng.gen_range(self.range.clone());
                ArithmeticProblem::new(left, self.operation, right)
            }
            Operation::Subtraction => {
                let left = rng.gen_range(self.range.clone());
                let right = rng.gen_range(*self.range.start()..=left);
                ArithmeticProblem::new(left, self.operation, right)
            }
            Operation::Multiplication => {
                let left = rng.gen_range(self.range.clone());
                let right = rng.gen_range(self.range.clone());
                ArithmeticProblem::new(left, self.operation, right)
            }
            Operation::Division => {
                let divisor = rng.gen_range(1..=9);
                let quotient = rng.gen_range(1..=9);
                ArithmeticProblem::new(divisor * quotient, self.operation, divisor)
            }
        }
    }
}

impl fmt::Display for ArithmeticProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.equation())
    }
}

pub fn boxed_generator(
    operation: Operation,
    range: RangeInclusive<i64>,
) -> Arc<dyn ProblemGenerator> {
    Arc::new(ArithmeticGenerator::new(operation, range))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn addition_stays_in_the_stage_range() {
        let generator = ArithmeticGenerator::new(Operation::Addition, 0..=9);
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..100 {
            let problem = generator.generate(&mut rng);
            assert!((0..=9).contains(&problem.left));
            assert!((0..=9).contains(&problem.right));
            assert_eq!(problem.answer, problem.left + problem.right);
        }
    }

    #[test]
    fn subtraction_never_produces_a_negative_answer() {
        let generator = ArithmeticGenerator::new(Operation::Subtraction, 0..=9);
        let mut rng = StdRng::seed_from_u64(8);
        for _ in 0..100 {
            let problem = generator.generate(&mut rng);
            assert!((0..=9).contains(&problem.left));
            assert!((0..=9).contains(&problem.right));
            assert!(problem.answer >= 0);
        }
    }

    #[test]
    fn multiplication_stays_in_the_stage_range() {
        let generator = ArithmeticGenerator::new(Operation::Multiplication, 0..=9);
        let mut rng = StdRng::seed_from_u64(9);
        for _ in 0..100 {
            let problem = generator.generate(&mut rng);
            assert!((0..=9).contains(&problem.left));
            assert!((0..=9).contains(&problem.right));
            assert_eq!(problem.answer, problem.left * problem.right);
        }
    }

    #[test]
    fn division_is_exact_and_never_divides_by_zero() {
        let generator = ArithmeticGenerator::new(Operation::Division, 0..=9);
        let mut rng = StdRng::seed_from_u64(10);
        for _ in 0..100 {
            let problem = generator.generate(&mut rng);
            assert!((1..=81).contains(&problem.left));
            assert!((1..=9).contains(&problem.right));
            assert_eq!(problem.left % problem.right, 0);
            assert_eq!(problem.answer, problem.left / problem.right);
        }
    }

    #[test]
    fn integer_answers_are_trimmed_and_checked() {
        let problem = ArithmeticProblem::new(3, Operation::Addition, 4);
        assert!(problem.is_correct(" 7 ", AnswerFormat::Integer));
        assert!(!problem.is_correct("7.0", AnswerFormat::Integer));
        assert_eq!(
            AnswerFormat::Integer.parse(""),
            Err(AnswerParseError::Empty)
        );
    }
}
