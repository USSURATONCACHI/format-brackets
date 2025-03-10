#![allow(incomplete_features)]
#![feature(iter_array_chunks, generic_const_exprs, ascii_char)]

use std::{io::{stdin, stdout, BufReader, BufWriter, Read, Write}, process::ExitCode, usize};

use cli::ArgsStructured;
use colored::Colorize;
use printer::Printer;
use regex::Regex;
use sliding_window::SlidingWindow;

pub mod cli;
pub mod sliding_window;
pub mod printer;

fn main() -> ExitCode {
    match inner_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        },
    }
}

fn pick_window_size(args: &ArgsStructured) -> usize {
    let min_plaintext_window_bytes = args.all_patterns_iterator()
        .filter_map(|x| x.plaintext())
        .map(|x| x.len())
        .max()
        .unwrap_or(1);

    let bytes_per_char = 4; // UTF8 allows at most 4 bytes per character.

    let window_size = min_plaintext_window_bytes
        .max(args.regex_window_size * bytes_per_char)
        .max(args.min_block_size * bytes_per_char);

    window_size
}

fn merge_min(val: &mut usize, with: Option<(usize, usize)>) {
    if let Some((with, _)) = with {
        *val = with.min(*val);
    }
}

fn inner_main() -> Result<(), String> {
    let args = cli::parse().map_err(|err| err.to_string())?;

    // == Setup
    let window_size = pick_window_size(&args) * 4; // 4 - arbitrary number. More = less performance overhead

    let reader = stdin();
    let reader = BufReader::new(reader);
    let mut reader = SlidingWindow::new(window_size * 2, reader);
    reader.fill().map_err(|x| x.to_string())?;

    let mut writer = BufWriter::new(stdout());
    let mut printer = Printer::new(args.min_block_size, writer); 


    // == Main loop
    let mut literal_stack = Vec::<usize>::new();
    let mut patterns_stack = Vec::<usize>::new();

    // let mut buf = vec![0u8; window_size].into_boxed_slice();
    let mut should_stop = false;
    while !should_stop {
        // let read = reader.read(&mut buf).map_err(|x| x.to_string())?;
        // if read == 0 {
        //     break;
        // }
        // let text = match str::from_utf8(&buf[..read]) {
        //     Ok(ok) => ok,
        //     Err(err) => unsafe { str::from_utf8_unchecked(&buf[..err.valid_up_to()]) },
        // };
        reader.fill().map_err(|x| x.to_string())?;
        let text = reader.get_window_utf8();
        if text.len() == 0 {
            break;
        }

        let (bytes_to_consume, is_closing) = single_iteration(&args, text, window_size, &mut literal_stack, &mut patterns_stack);

        // if literal_stack.len() > 0 {
        //     writer.write(format!(" {} ", bytes_to_consume).on_blue().to_string().as_bytes()).unwrap();
        // } else if patterns_stack.len() > 0 {
        //     writer.write(format!(" {} ", bytes_to_consume).on_yellow().to_string().as_bytes()).unwrap();
        // } else {
        //     writer.write(format!(" {} ", bytes_to_consume).on_bright_white().to_string().as_bytes()).unwrap();
        // }

        let is_literal = literal_stack.len() > 0;
        let indent_level = patterns_stack.len();

        let mut buf = [0u8; 1024];
        let mut total_consumed = 0;
        while total_consumed < bytes_to_consume {
            let to_read = bytes_to_consume.min(buf.len());
            let was_read = reader.read(&mut buf[..to_read]).map_err(|x| x.to_string())?;
            if was_read == 0 {
                should_stop = true;
                break;
            }
            total_consumed += was_read;

            printer.push_segment(
                &buf[..was_read], 
                indent_level, 
                is_literal,
                is_closing
            ).map_err(|x| x.to_string())?;

            // write_escaped_newlines(&mut writer, &buf[..was_read]).map_err(|x| x.to_string())?;
            // writer.write("\n".as_bytes()).map_err(|x| x.to_string())?;
        }
        printer.writer().flush().unwrap();
    }   
    
    Ok(())
}

/// Returns count of bytes that should be consumed
fn single_iteration(
    args: &ArgsStructured, text: &str, window_size: usize, 
    literal_stack: &mut Vec<usize>, patterns_stack: &mut Vec<usize>
) -> (usize, bool) {
    // print!("Text is {}:", text.len());
    // write_escaped_newlines(&mut stdout(), text.as_bytes()).unwrap();
    // print!("\n");
    let last_literal = literal_stack.last().cloned();
    let last_pattern = patterns_stack.last().cloned();

    let mut max_possible_jump_bytes = text.len();

    if let Some(literal_idx) = last_literal {
        // Check for current literal closing
        let closing_literal = &args.literals[literal_idx].1;
        let pos = closing_literal.find_in(text);
        merge_min(&mut max_possible_jump_bytes, pos);

        if let Some((0, end)) = pos {
            // Literal closed
            literal_stack.pop();
            return (end, false);
        }

    } else if let Some(pattern_idx) = last_pattern {
        // Check for current pattern closing
        let closing_pattern = &args.patterns[pattern_idx].1;
        let pos = closing_pattern.find_in(text);
        merge_min(&mut max_possible_jump_bytes, pos);

        if let Some((0, end)) = pos {
            // Pattern closed
            patterns_stack.pop();
            return (end, true);
        }

    }

    // Check for patterns opening
    if literal_stack.is_empty() {
        for (idx, (opening_pattern, _)) in args.patterns.iter().enumerate() {
            let pos = opening_pattern.find_in(text);
            merge_min(&mut max_possible_jump_bytes, pos);

            if let Some((0, end)) = pos {
                // Pattern opened
                patterns_stack.push(idx);
                return (end, false);
            }

        }
    }

    // Check for literals opening
    for (idx, (opening_literal, _)) in args.literals.iter().enumerate() {
        let pos = opening_literal.find_in(text);
        merge_min(&mut max_possible_jump_bytes, pos);

        if let Some((0, end)) = pos {
            // Literal opened
            literal_stack.push(idx);
            return (end, false);
        }

    }

    // println!("Max possible jump is: {}", max_possible_jump_bytes);
    return (max_possible_jump_bytes, false);
}

fn write_escaped_newlines(writer: &mut impl Write, data: &[u8]) -> std::io::Result<usize> {
    let newline = "↲".blue().to_string();
    let mut was_written = 0;

    let mut is_first = true; 
    for part in data.split(|x| *x == '\n'.as_ascii().unwrap().to_u8()) {
        if !is_first {
            was_written += writer.write(newline.as_bytes())?;
        }
        is_first = false;
        
        was_written += writer.write(part)?;
    }

    Ok(was_written)
}