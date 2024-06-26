use std::str::Chars;

/// Peekable iterator over a string.
///
/// first() to peek
/// next() to advance
pub struct Cursor<'a> {
    chars: Chars<'a>,
    pos: usize,
    prev: char,
}

pub(crate) const EOF_CHAR: char = '\0';

// Anything that refers to Chars will need to refer to the lifetime <'a>
impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            chars: input.chars(),
            pos: 0,
            prev: EOF_CHAR,
        }
    }

    /// Returns the previous character that was at the cursor.
    pub(crate) fn prev(&self) -> char {
        self.prev
    }

    /// Peeks at the next character in the input without advancing the cursor.
    pub fn first(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    /// Peeks at the second character in the input without advancing the cursor.
    pub(crate) fn second(&self) -> char {
        let mut chars = self.chars.clone();
        chars.next();
        chars.next().unwrap_or(EOF_CHAR)
    }
    
    pub(crate) fn reset_pos(&mut self) {
        self.pos = 0;
    }

    /// Advances the cursor by one character and returns the character that was
    /// at the cursor before advancing.
    pub(crate) fn next(&mut self) -> char {
        let ch = self.chars.next().unwrap_or(EOF_CHAR);

        #[cfg(debug_assertions)]
        {
            self.prev = ch;
        }

        self.pos += 1;
        ch
    }

    /// Eats characters while the predicate returns true.
    pub(crate) fn eat_while(
        &mut self, 
        mut predicate: impl FnMut(char) -> bool
    ) -> String {
        let mut eaten: String = self.prev().to_string();

        while predicate(self.first()) && self.first() != EOF_CHAR {
            eaten.push(self.first());
            self.next();
        }

        eaten
    }
}
