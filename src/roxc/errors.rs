use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::Config;
use lalrpop_util::lexer::Token;
use lalrpop_util::{ErrorRecovery, ParseError};
use std::fs::read_to_string;
use std::path::PathBuf;

use super::{Span, Spanned};

/// Rox's custom `Result` type
pub type Result<T> = std::result::Result<T, RoxError>;

fn clean_expected(expected: &[String]) -> Vec<String> {
    expected
        .iter()
        .map(|string| {
            string
                .chars()
                .skip(1)
                .take(string.len() - 2)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
}

#[derive(Clone, Debug)]
pub struct RoxError {
    pub file: SimpleFile<String, String>,
    pub message: Option<String>,
    pub labels: Vec<Label<()>>,
    pub notes: Vec<String>,
}

impl RoxError {
    pub fn new<T: Clone + Into<PathBuf>>(file: T, message: &str) -> Self {
        let contents = read_to_string(file.clone().into()).unwrap();
        RoxError {
            file: SimpleFile::new(
                file.into().to_str().unwrap().to_string(),
                contents,
            ),
            message: Some(message.to_string()),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_label(self, message: &str, span: Span) -> RoxError {
        let mut labels = self.labels;
        labels.push(Label::primary((), span.0..span.1).with_message(message));
        RoxError { labels, ..self }
    }

    pub fn with_file_placeholder(message: &str) -> Self {
        let path = PathBuf::from("./scratch/test.rox");
        Self::new(path, message)
    }

    pub fn from_error_recoveries<T: Clone + Into<PathBuf>>(
        error_vec: Vec<
            Spanned<
                lalrpop_util::ErrorRecovery<
                    usize,
                    lalrpop_util::lexer::Token<'_>,
                    &'static str,
                >,
            >,
        >,
        file: T,
    ) -> Self {
        let rox_errors: Vec<RoxError> = error_vec
            .iter()
            .map(
                |Spanned {
                     value: ErrorRecovery { error, .. },
                     span,
                 }| {
                    RoxError::from_parse_error(
                        Spanned {
                            span: span.clone(),
                            value: error.clone(),
                        },
                        file.clone().into(),
                    )
                },
            )
            .collect();
        let labels = rox_errors
            .iter()
            .flat_map(|RoxError { labels, .. }| labels.clone())
            .collect::<Vec<_>>();
        RoxError {
            file: SimpleFile::new(
                file.clone().into().to_str().unwrap().to_string(),
                read_to_string(file.into()).unwrap(),
            ),
            message: Some(String::from(
                "Encountered the following errors while parsing",
            )),
            notes: vec![],
            labels,
        }
    }

    pub fn from_parse_error(
        error: Spanned<ParseError<usize, Token<'_>, &'static str>>,
        file: PathBuf,
    ) -> Self {
        let file_source = read_to_string(file.clone()).unwrap();
        let file =
            SimpleFile::new(String::from(file.to_str().unwrap()), file_source);
        match error.value {
            ParseError::InvalidToken { .. } => RoxError {
                file,
                message: None,
                labels: vec![Label::primary((), error.span.0..error.span.1)
                    .with_message("Invalid token.")],
                notes: Vec::new(),
            },
            ParseError::UnrecognizedEOF { expected, .. } => RoxError {
                file,
                message: None,
                labels: vec![Label::primary((), error.span.0..error.span.1)
                    .with_message("Unrecognized EOF")],
                notes: vec![format!(
                    "Expected one of {:?}",
                    clean_expected(&expected)
                )],
            },
            ParseError::UnrecognizedToken {
                token: (start, _token, end),
                expected,
            } => RoxError {
                file,
                message: None,
                labels: vec![Label::primary((), start..end)
                    .with_message("Unrecognized token ")],
                notes: vec![format!(
                    "Expected one of {:?}",
                    clean_expected(&expected)
                )],
            },
            ParseError::ExtraToken {
                token: (start, token, end),
            } => RoxError {
                file,
                message: Some(format!("Encountered extra token: {:?}", token)),
                labels: vec![Label::primary((), start..end)],
                notes: Vec::new(),
            },
            // Customer user error message
            // Rox doesn't emit these yet
            ParseError::User { error } => RoxError {
                file,
                message: Some(String::from(error)),
                labels: Vec::new(),
                notes: Vec::new(),
            },
        }
    }

    pub fn emit_error(&self) -> std::io::Result<()> {
        let mut diagnostic: Diagnostic<()> = Diagnostic::error()
            .with_labels(self.labels.clone())
            .with_notes(self.notes.clone());
        if let Some(message) = self.message.as_ref() {
            diagnostic = diagnostic.clone().with_message(message);
        }

        let mut writer = StandardStream::stderr(ColorChoice::Always);
        let config = Config::default();

        codespan_reporting::term::emit(
            &mut writer,
            &config,
            &self.file,
            &diagnostic,
        )
    }
}
