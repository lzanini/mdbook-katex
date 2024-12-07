//! Scan Markdown text and identify math block events.
use super::*;

/// A pair of strings are delimiters.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Delimiter {
    /// Left delimiter.
    pub left: String,
    /// Right delimiter.
    pub right: String,
}

impl Delimiter {
    /// Same left and right `delimiter`.
    pub fn same(delimiter: String) -> Self {
        Self {
            left: delimiter.clone(),
            right: delimiter,
        }
    }

    /// The first byte of the left delimiter.
    pub fn first(&self) -> u8 {
        self.left.as_bytes()[0]
    }

    /// Whether `to_match` matches the left delimiter.
    pub fn match_left(&self, to_match: &[u8]) -> bool {
        if self.left.len() > to_match.len() {
            return false;
        }
        for (we, they) in self.left.as_bytes().iter().zip(to_match) {
            if we != they {
                return false;
            }
        }
        true
    }
}

/// An event for parsing in a Markdown file.
#[derive(Debug)]
pub enum Event {
    /// A beginning of text or math block.
    Begin(usize),
    /// An end of a text block.
    TextEnd(usize),
    /// An end of an inline math block.
    InlineEnd(usize),
    /// An end of a display math block.
    BlockEnd(usize),
}

/// Scanner for text to identify block and inline math `Event`s.
#[derive(Debug)]
pub struct Scan<'a> {
    string: &'a str,
    bytes: &'a [u8],
    index: usize,
    /// Buffer for block and inline math `Event`s.
    pub events: VecDeque<Event>,
    block_delimiter: &'a Delimiter,
    inline_delimiter: &'a Delimiter,
}

impl Iterator for Scan<'_> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.events.pop_front() {
                Some(item) => return Some(item),
                None => self.process_byte().ok()?,
            }
        }
    }
}

impl<'a> Scan<'a> {
    /// Set up a `Scan` for `string` with given delimiters.
    pub fn new(
        string: &'a str,
        block_delimiter: &'a Delimiter,
        inline_delimiter: &'a Delimiter,
    ) -> Self {
        Self {
            string,
            bytes: string.as_bytes(),
            index: 0,
            events: VecDeque::new(),
            block_delimiter,
            inline_delimiter,
        }
    }

    /// Scan, identify and store all `Event`s in `self.events`.
    pub fn run(&mut self) {
        while let Ok(()) = self.process_byte() {}
    }

    /// Get byte currently pointed to. Returns `Err(())` if out of bound.
    fn get_byte(&self) -> Result<u8, ()> {
        self.bytes.get(self.index).map(|b| b.to_owned()).ok_or(())
    }

    /// Increment index.
    fn inc(&mut self) {
        self.index += 1;
    }

    /// Scan one byte, proceed process based on the byte.
    /// - Start of delimiter => call `process_delimit`.
    /// - `\` => skip one byte.
    /// - `` ` `` => call `process_backtick`.
    ///     Return `Err(())` if no more bytes to process.
    fn process_byte(&mut self) -> Result<(), ()> {
        let byte = self.get_byte()?;
        self.inc();
        match byte {
            b if b == self.block_delimiter.first()
                && self
                    .block_delimiter
                    .match_left(&self.bytes[(self.index - 1)..]) =>
            {
                self.index -= 1;
                self.process_delimit(false)?;
            }
            b if b == self.inline_delimiter.first()
                && self
                    .inline_delimiter
                    .match_left(&self.bytes[(self.index - 1)..]) =>
            {
                self.index -= 1;
                self.process_delimit(true)?;
            }
            b'\\' => {
                self.inc();
            }
            b'`' => self.process_backtick()?,
            _ => (),
        }
        Ok(())
    }

    /// Fully skip a backtick-delimited code block.
    /// Guaranteed to match the number of backticks in delimiters.
    /// Return `Err(())` if no more bytes to process.
    fn process_backtick(&mut self) -> Result<(), ()> {
        let mut n_back_ticks = 1;
        loop {
            let byte = self.get_byte()?;
            if byte == b'`' {
                self.inc();
                n_back_ticks += 1;
            } else {
                break;
            }
        }
        loop {
            self.index += self.string[self.index..]
                .find(&"`".repeat(n_back_ticks))
                .ok_or(())?
                + n_back_ticks;
            if self.get_byte()? == b'`' {
                // Skip excessive backticks.
                self.inc();
                while let b'`' = self.get_byte()? {
                    self.inc();
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Skip a full math block.
    /// Add `Event`s to mark the start and end of the math block and
    /// surrounding text blocks.
    /// Return `Err(())` if no more bytes to process.
    fn process_delimit(&mut self, inline: bool) -> Result<(), ()> {
        if self.index > 0 {
            self.events.push_back(Event::TextEnd(self.index));
        }

        let delim = if inline {
            self.inline_delimiter
        } else {
            self.block_delimiter
        };
        self.index += delim.left.len();
        self.events.push_back(Event::Begin(self.index));

        loop {
            self.index += self.string[self.index..].find(&delim.right).ok_or(())?;

            // Check `\`.
            let mut escaped = false;
            let mut checking = self.index;
            loop {
                checking -= 1;
                if self.bytes.get(checking) == Some(&b'\\') {
                    escaped = !escaped;
                } else {
                    break;
                }
            }
            if !escaped {
                let end_event = if inline {
                    Event::InlineEnd(self.index)
                } else {
                    Event::BlockEnd(self.index)
                };
                self.events.push_back(end_event);
                self.index += delim.right.len();
                self.events.push_back(Event::Begin(self.index));
                break;
            } else {
                self.index += delim.right.len();
            }
        }

        Ok(())
    }
}
