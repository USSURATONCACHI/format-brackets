use std::io::Write;

pub struct Printer<P: Write> {
    min_block_size: usize,
    was_literal: bool,
    is_at_line_start: bool,
    prev_indent: usize,
    output: P,
}

impl<P: Write> Printer<P> {
    pub fn new(min_block_size: usize, output: P) -> Self {
        Self { 
            min_block_size,
            was_literal: false,
            is_at_line_start: true,
            prev_indent: 0,
            output
        }
    }

    pub fn push_segment(&mut self, segment: &[u8], indentation_level: usize, is_literal: bool, is_closing: bool) -> std::io::Result<()> {
        self.push_naive(segment, indentation_level, is_literal, is_closing)
    }

    fn write(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        const NEWLINE: u8 = '\n'.as_ascii().unwrap().to_u8();

        self.output.write_all(bytes)?;
        self.is_at_line_start = bytes.last().cloned() == Some(NEWLINE);
        Ok(())
    }

    fn push_naive(&mut self, segment: &[u8], indentation_level: usize, is_literal: bool, is_closing: bool) -> std::io::Result<()> {
        if segment.is_empty() {
            return Ok(());
        }
        
        if is_literal {
            if self.is_at_line_start && !self.was_literal {
                for _ in 0..indentation_level {
                    self.write("\t".as_bytes())?;
                }
            }

            self.write(segment)?;
        } else {
            let newline: u8 = '\n'.as_ascii().unwrap().to_u8();


            let mut is_first_line = true;
            for part in segment.split(|x| *x == newline) {
                let line = unsafe { str::from_utf8_unchecked(part) };
                let line = if is_only_whitespace(line) {
                    line
                } else {
                    line.trim_start()
                };

                if self.prev_indent != indentation_level && !self.is_at_line_start && is_closing {
                    self.write("\n".as_bytes())?;
                }
                
                if !is_first_line {
                    self.write("\n".as_bytes())?;
                }
                is_first_line = false;

                if line.len() > 0 {
                    if self.is_at_line_start {
                        for _ in 0..indentation_level {
                            write!(self.output, "\t")?;
                        }
                    }
                    self.write(line.as_bytes())?;
                }

                if self.prev_indent != indentation_level && !self.is_at_line_start && !is_closing {
                    self.write("\n".as_bytes())?;
                }
            }
        }

        self.was_literal = is_literal;
        self.prev_indent = indentation_level;

        Ok(())
    }

    pub fn writer(&mut self) -> &mut P {
        &mut self.output
    }
}

fn is_only_whitespace(text: &str) -> bool {
    text.chars().all(|c| c.is_whitespace())
}