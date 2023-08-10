use crate::html_parser::token_replacements::TOKEN_REPLACEMENTS;
use crate::html_parser::tokenizer::{Tokenizer};

// Consumes a character reference and places this in the tokenizer consume buffer
pub fn consume_character_reference(&mut tokenizer: Tokenizer, additional_allowed_char: Option<char>) {
    let c = match tokenizer.stream.read_char() {
        Ok(c) => c,
        Err(_) => {
            tokenizer.clear_consume_buffer();
            return;
        }
    };

    // If we allow an extra character, check for it
    if additional_allowed_char.is_some() && c == additional_allowed_char.unwrap() {
        tokenizer.stream.unread();
        tokenizer.clear_consume_buffer();
        return
    }

    match c {
        CHAR_TAB | CHAR_LF | CHAR_FF => return,
        '#' => consume_dash_entity(tokenizer),
        _ => consume_anything_else(tokenizer),
    }
}

// Consume a dash entity #x1234, #123 etc
fn consume_dash_entity(&mut tokenizer: Tokenizer) {
    let mut str_num = "";

    // Save length for easy recovery
    let len = tokenizer.get_consume_len();

    // Consume the dash
    tokenizer.consume('#');

    // Is the char a 'X' or 'x', then we must fetch hex digits
    let mut is_hex = false;
    let hex = tokenizer.stream.look_ahead(1);
    if hex == 'x' || hex == 'X' {
        is_hex = true;
        // Consume the 'x' character
        let c = tokenizer.stream.read_char();
        tokenizer.consume(c);
    }

    let mut i = 0;
    loop {
        let (c, eof) = tokenizer.stream.read_char();
        if eof {
            tokenizer.set_consume_len(len);
            return
        }

        if is_hex && c.is_ascii_hexdigit() {
            str_num.push(c);
            tokenizer.consume(c);
        } else if !is_hex && c.is_ascii_digit() {
            str_num.push(c);
            tokenizer.consume(c);
        } else {
            break;
        }

        i += 1;
    }

    // Fetch next character
    let (c, eof) = tokenizer.stream.read_char();
    if eof {
        tokenizer.set_consume_len(len);
        return
    }

    // Next character MUST be ;
    if c != ';' {
        tokenizer.parse_error("expected a ';'");
        tokenizer.set_consume_len(len);
        return
    }

    // If we found ;. we need to check how many digits we have parsed. It needs to be at least 1,
    if i == 0 {
        tokenizer.parse_error("didn't expect #;");
        tokenizer.set_consume_len(len);
        return
    }

    // check if we need to replace the character. First convert the number to a uint, and use that
    // to check if it exists in the replacements table.
    let num = match u32::from_str_radix(str_num, if is_hex { 16 } else { 10 }) {
        Ok(value) => value,
        Err(_) => 0,    // lets pretend that an invalid value is set to 0
    };

    if TOKEN_REPLACEMENTS.contains_key(&num) {
        tokenizer.set_consume_len(len);
        tokenizer.consume(*TOKEN_REPLACEMENTS.get(&num).unwrap());
        return;
    }

    // Next, check if we are in the 0xD800..0xDFFF or 0x10FFFF range, if so, replace
    if (num > 0xD800 && num < 0xDFFF) || (num > 0x10FFFFF) {
        tokenizer.set_consume_len(len);
        tokenizer.parse_error("within reserved codepoint range, but replaced");
        tokenizer.consume(Tokenizer::CHAR_REPLACEMENT);
    }

    // Check if it's in a reserved range, in that case, we ignore the data
    if in_reserved_number_range(num) {
        tokenizer.set_consume_len(len);
        tokenizer.parse_error("within reserved codepoint range, ignored");
    }
}

// Returns if the given codepoint number is in a reserved range (as defined in
// https://dev.w3.org/html5/spec-LC/tokenization.html#consume-a-character-reference)
fn in_reserved_number_range(codepoint: u32) -> bool {
    if
        (0x0001..=0x0008).contains(&codepoint) ||
        (0x000E..=0x001F).contains(&codepoint) ||
        (0x007F..=0x009F).contains(&codepoint) ||
        (0xFDD0..=0xFDEF).contains(&codepoint) ||
        (0x000E..=0x001F).contains(&codepoint) ||
        (0x000E..=0x001F).contains(&codepoint) ||
        (0x000E..=0x001F).contains(&codepoint) ||
        (0x000E..=0x001F).contains(&codepoint) ||
        (0x000E..=0x001F).contains(&codepoint) ||
        [
            0x000B, 0xFFFE, 0xFFFF, 0x1FFFE, 0x1FFFF, 0x2FFFE, 0x2FFFF, 0x3FFFE, 0x3FFFF,
            0x4FFFE, 0x4FFFF, 0x5FFFE, 0x5FFFF, 0x6FFFE, 0x6FFFF, 0x7FFFE, 0x7FFFF,
            0x8FFFE, 0x8FFFF, 0x9FFFE, 0x9FFFF, 0xAFFFE, 0xAFFFF, 0xBFFFE, 0xBFFFF,
            0xCFFFE, 0xCFFFF, 0xDFFFE, 0xDFFFF, 0xEFFFE, 0xEFFFF, 0xFFFFE, 0xFFFFF,
            0x10FFFE, 0x10FFFF
        ].contains(&codepoint) {
        return true;
    }

    return false;
}

// This will consume any other matter that does not start with &# (ie: &raquo; &#copy;)
fn consume_anything_else(&mut tokenizer: Tokenizer) {

}