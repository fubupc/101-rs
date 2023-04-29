use std::{io, ops::Index};

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Options {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    pub stem: String,
    pub option_a: String,
    pub option_b: String,
    pub option_c: String,
    pub option_d: String,
    pub key: Options,
}

impl Index<Options> for Question {
    type Output = String;
    fn index(&self, index: Options) -> &Self::Output {
        match index {
            Options::A => &self.option_a,
            Options::B => &self.option_b,
            Options::C => &self.option_c,
            Options::D => &self.option_d,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quiz {
    questions: Vec<Question>,
}

impl Quiz {
    pub fn new() -> Quiz {
        Quiz {
            questions: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.questions.len()
    }

    pub fn add_question(&mut self, q: Question) {
        self.questions.push(q)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (usize, &'a Question)> {
        self.questions
            .iter()
            .enumerate()
            .map(|(idx, q)| (idx + 1, q))
    }

    pub fn new_answers<'a>(&'a self) -> Answers<'a> {
        Answers {
            quiz: self,
            answers: vec![None; self.questions.len()],
        }
    }
}

pub struct Answers<'a> {
    quiz: &'a Quiz,
    answers: Vec<Option<Options>>,
}

impl<'a> Answers<'a> {
    pub fn choose(&mut self, num: usize, choice: Options) -> Result<(), anyhow::Error> {
        if num < 1 || num > self.quiz.len() {
            anyhow::bail!("Invalid question number")
        }
        self.answers[num - 1] = Some(choice);
        Ok(())
    }

    pub fn score(&self) -> Score {
        let correct = self
            .quiz
            .iter()
            .filter(|(num, q)| self.answers[num - 1] == Some(q.key))
            .count();

        Score {
            total: self.quiz.len(),
            correct,
        }
    }
}

pub struct Score {
    pub total: usize,
    pub correct: usize,
}

// impl Serialize for Quiz {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut quiz = serializer.serialize_struct("Quiz", 1)?;
//         quiz.serialize_field("questions", &self.questions)?;
//         quiz.end()
//     }
// }

pub fn save_quiz<W: io::Write>(w: W, q: &Quiz) -> serde_json::Result<()> {
    serde_json::to_writer_pretty(w, q)
}

pub fn load_quiz<R: io::Read>(r: R) -> serde_json::Result<Quiz> {
    serde_json::from_reader(r)
}

#[cfg(test)]
mod tests {
    use crate::Options;

    #[test]
    fn test_serde() {
        println!("{:?}", serde_json::to_string(&Options::A));
    }
}
