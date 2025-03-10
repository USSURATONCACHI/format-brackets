use clap::{command, Parser};
use regex::Regex;
use colored::Colorize;

#[derive(Parser, Debug)]
#[command(
    author = "ussur", 
    version = "1.0", 
    about = "Reads `stdin` and formats it based on patterns (brackets by default) to make more readable"
)]
pub struct Args {
    #[arg(
        short = 'p', long, num_args = 2, 
        value_names = ["opening_pattern", "closing_pattern"], 
        help = "List of pairs of plain-text patterns to format against",
        default_values = &["(", ")", "[", "]", "{", "}"],
    )]
    patterns: Vec<String>,

    #[arg(
        short = 'r', long, num_args = 2, 
        value_names = ["opening_pattern", "closing_pattern"],
        help = "List of pairs of regex patterns to format against"
    )]
    patterns_regex: Vec<String>,

    #[arg(
        short = 'l', long, num_args = 2, 
        value_names = ["opening_pattern", "closing_pattern"], 
        help = "List of pairs of plain-text patterns of string literals, formatting in which will be ignored",
        default_values = &["\"", "\"", "«", "»", "'", "'", "„", "“", "//", "\n", "/*", "*/", "#=", "=#", "#", "\n"],
    )]
    literals: Vec<String>,

    #[arg(
        visible_alias = "lr", long, num_args = 2, 
        value_names = ["opening_pattern", "closing_pattern"],
        help = "List of pairs of regex patterns of string literals, formatting in which will be ignored",
    )]
    literals_regex: Vec<String>,

    #[arg(short, long, help = "Size of the minimum block to be formatted", default_value = "20")]
    min_block_size: usize,

    #[arg(long, help = "Size of the window for regexes to be applied to", default_value = "100")]
    regex_window_size: usize,

    #[arg(short = 's', long, help = "Flag to NOT ignore patterns that are prepended by an escape sequence")]
    disallow_escaping: bool,

    #[arg(short = 'e', long, help = "Patterns (brackets) prepended with this will be ignored (escape sequences will be left unchanged)", default_value = "\\")]
    escape_sequence: String,
    
}

#[derive(Debug, Clone)]
pub enum Pattern {
    PlainText(String),
    Regex(Regex),
}

impl Pattern {
    pub fn find_in(&self, text: &str) -> Option<(usize, usize)> {
        match self {
            Pattern::PlainText(pat) =>
                text.find(pat).map(|start| (start, start + pat.len())),

            Pattern::Regex(regex) =>
                regex.find(text).map(|mat| (mat.start(), mat.end())),
        }
    } 
}

impl Pattern {
    pub fn is_plaintext(&self) -> bool {
        self.plaintext().is_some()
    }
    pub fn plaintext(&self) -> Option<&String> {
        if let Pattern::PlainText(text) = self {
            Some(text)
        } else {
            None
        }
    }
    pub fn is_regex(&self) -> bool {
        self.regex().is_some()
    }
    pub fn regex(&self) -> Option<&Regex> {
        if let Pattern::Regex(regex) = self {
            Some(regex)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArgsStructured {
    pub patterns: Vec<(Pattern, Pattern)>,
    pub literals: Vec<(Pattern, Pattern)>,
    pub min_block_size: usize,
    pub regex_window_size: usize,
    pub disallow_escaping: bool,
    pub escape_sequence: String,
}

impl ArgsStructured {
    pub fn all_patterns_iterator<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = &'a Pattern>> {
        let iter = self.patterns.iter()
            .chain(self.literals.iter())
            .map(
                |(a, b)| [a, b].into_iter()
            )
            .flatten();
    
        Box::new(iter)
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    RegexesNotCompiled(Vec<(String, regex::Error)>),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::RegexesNotCompiled(items) => {
                let mut string = "".to_owned(); 
                string.push_str("Failed to compile all regexes.\n".red().to_string().as_str());

                for (text, error) in items {
                    string.push_str(&format!("\nRegex: '{}'\n", text.blue()));
                    match error {
                        regex::Error::Syntax(error) => {
                            string.push_str(&format!("{} - {}", "Syntax error".red(), error));
                        },
                        regex::Error::CompiledTooBig(max_size) => {
                            string.push_str(&format!("{} - Internal representation of regex exceeded {} bytes", "Compiled too big".red(), max_size));
                        },
                        other => {
                            string.push_str(&format!("{} - {:?}", "Unknown error".red(), other));
                        },
                    }
                }

                string
            },
        }
    }
}

pub fn parse() -> Result<ArgsStructured, Error> {
    structure(Args::parse())
}

pub fn structure(args: Args) -> Result<ArgsStructured, Error> {
    let mut regex_errors= vec![]; 

    // Patterns
    let mut patterns: Vec<(Pattern, Pattern)> = Vec::with_capacity(args.patterns.len() / 2 + args.patterns_regex.len() / 2);
    {
        // Plaintext
        let plaintext = args.patterns.into_iter()
            .map(|x| Pattern::PlainText(x))
            .array_chunks::<2>()
            .map(|[a, b]| (a, b));
        patterns.extend(plaintext);


        let regexes =  args.patterns_regex.into_iter()
            .filter_map(
                |text| {
                    match Regex::new(&text) {
                        Ok(r) => Some(r),
                        Err(err) => {
                            regex_errors.push((text, err));
                            None
                        },
                    }
                }
            );

        // Regexes
        for [open, close] in regexes.array_chunks::<2>() {
            let pair = (Pattern::Regex(open), Pattern::Regex(close));
            patterns.push(pair);
        }
    }
    let patterns = patterns;

    // Literals
    let mut literals: Vec<(Pattern, Pattern)> = Vec::with_capacity(args.literals.len() / 2 + args.literals_regex.len() / 2);
    {
        let plaintext = args.literals.into_iter()
            .map(|x| Pattern::PlainText(x))
            .array_chunks::<2>()
            .map(|[a, b]| (a, b));
        literals.extend(plaintext);

        let regexes =  args.literals_regex.into_iter()
            .filter_map(
                |text| {
                    match Regex::new(&text) {
                        Ok(r) => Some(r),
                        Err(err) => {
                            regex_errors.push((text, err));
                            None
                        },
                    }
                }
            )
            .map(|x| Pattern::Regex(x))
            .array_chunks::<2>()
            .map(|[a, b]| (a, b));
        literals.extend(regexes);
    }
    let literals = literals;

    if !regex_errors.is_empty() {
        return Err(Error::RegexesNotCompiled(regex_errors));
    }

    Ok(
        ArgsStructured {
            patterns,
            literals,
            min_block_size: args.min_block_size,
            regex_window_size: args.regex_window_size,
            disallow_escaping: args.disallow_escaping,
            escape_sequence: args.escape_sequence,
        }
    )
} 