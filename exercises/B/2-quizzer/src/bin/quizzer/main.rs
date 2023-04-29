use std::fs::File;
use std::io::{stderr, stdout};

use crossterm::execute;
use crossterm::style::{
    Attribute, Color, Print, PrintStyledContent, SetAttribute, SetForegroundColor, Stylize,
};
use derive_more::Display;
use inquire::{InquireError, Select, Text};
use quizzer::{Answers, Options, Quiz};

#[derive(Display)]
enum Mode {
    QuestionEntering,
    Quiz,
}

fn main() {
    match run() {
        Err(err) => print_error(&err.to_string()),
        _ => {}
    }
}

fn run() -> anyhow::Result<()> {
    let mode = Select::new("Select mode:", vec![Mode::QuestionEntering, Mode::Quiz]).prompt()?;

    match mode {
        Mode::QuestionEntering => {
            let quiz = enter_questions()?;
            save_to_file(&quiz)?;
        }
        Mode::Quiz => {
            let quiz = load_from_file()?;
            let answers = take_quiz(&quiz)?;
            let score = answers.score();
            println!("Final score: {}/{}", score.correct, score.total);
        }
    }
    Ok(())
}

fn enter_questions() -> Result<Quiz, InquireError> {
    let mut quiz = Quiz::new();
    for num in 1.. {
        println!("Add question #{num}:");
        quiz.add_question(quizzer::Question {
            stem: Text::new("Stem:").prompt()?,
            option_a: Text::new("Option A:").prompt()?,
            option_b: Text::new("Option B:").prompt()?,
            option_c: Text::new("Option C:").prompt()?,
            option_d: Text::new("Option D:").prompt()?,
            key: Select::new("Key:", vec![Options::A, Options::B, Options::C, Options::D])
                .prompt()?,
        });
        match Select::new("Continue to add question:", vec!["Yes", "No"]).prompt() {
            Ok("Yes") => continue,
            _ => break,
        };
    }
    Ok(quiz)
}

fn take_quiz(quiz: &Quiz) -> Result<Answers, InquireError> {
    let mut answers = quiz.new_answers();
    for (num, question) in quiz.iter() {
        execute!(
            stdout(),
            PrintStyledContent(format!("Question {}/{}: ", num, quiz.len()).bold()),
            Print(&question.stem),
            Print("\n"),
            PrintStyledContent("A: ".bold()),
            Print(&question.option_a),
            Print("\n"),
            PrintStyledContent("B: ".bold()),
            Print(&question.option_b),
            Print("\n"),
            PrintStyledContent("C: ".bold()),
            Print(&question.option_c),
            Print("\n"),
            PrintStyledContent("D: ".bold()),
            Print(&question.option_d),
            Print("\n"),
        )?;

        let choosed = loop {
            match Select::new(
                "Choose an answer:",
                vec![Options::A, Options::B, Options::C, Options::D],
            )
            .prompt()
            {
                Ok(opt) => break opt,
                Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
                    match Select::new("Are you sure to quit?", vec!["Yes", "No"]).prompt() {
                        Ok("Yes") => return Err(InquireError::OperationInterrupted),
                        Ok(_)
                        | Err(InquireError::OperationCanceled)
                        | Err(InquireError::OperationInterrupted) => continue,
                        Err(err) => return Err(err),
                    }
                }
                Err(err) => return Err(err),
            }
        };

        answers.choose(num, choosed).unwrap();
    }

    Ok(answers)
}

fn load_from_file() -> anyhow::Result<Quiz> {
    let input = loop {
        let input = Text::new("Select a quiz file:").prompt()?;
        match File::open(input.trim()) {
            Ok(out) => break out,
            Err(err) => {
                print_error(&format!("Cannot open file: {err}"));
            }
        }
    };
    quizzer::load_quiz(&input).map_err(|e| e.into())
}

fn save_to_file(quiz: &Quiz) -> anyhow::Result<()> {
    let out = loop {
        let out = Text::new("Select an output file to save quiz:").prompt()?;
        match File::create(out.trim()) {
            Ok(out) => break out,
            Err(err) => {
                print_error(&format!("Cannot create file: {err}"));
            }
        }
    };

    quizzer::save_quiz(&out, quiz)?;
    out.sync_all()?;
    print_ok(&format!("Quiz saved to file: {}", "abc"));
    Ok(())
}

fn print_error(err: &str) {
    execute!(
        stderr(),
        SetForegroundColor(Color::Red),
        SetAttribute(Attribute::Bold),
        Print("Error:"),
        SetAttribute(Attribute::Reset),
        Print(format!(" {err}\n")),
    )
    .unwrap()
}

fn print_ok(msg: &str) {
    execute!(
        stdout(),
        SetForegroundColor(Color::Green),
        SetAttribute(Attribute::Bold),
        Print("Success:"),
        SetAttribute(Attribute::Reset),
        Print(format!(" {msg}\n")),
    )
    .unwrap()
}
