//! Minimal RFC 4180 reader, enough for the collection exports members attach.
//!
//! Card names hold commas and apostrophes, so quoted fields have to be honoured
//! rather than splitting the lines on commas.

/// Splits a CSV document into records of fields.
pub fn parse(content: &str) -> Vec<Vec<String>> {
    let mut records = Vec::new();
    let mut record = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut characters = content.chars().peekable();

    while let Some(character) = characters.next() {
        match character {
            '"' if quoted => {
                // Inside a quoted field a doubled quote stands for one quote.
                if characters.peek() == Some(&'"') {
                    characters.next();
                    field.push('"');
                } else {
                    quoted = false;
                }
            }
            // Quotes only open a field, elsewhere they are part of the value.
            '"' if field.is_empty() => quoted = true,
            ',' if !quoted => record.push(std::mem::take(&mut field)),
            '\n' if !quoted => {
                record.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut record));
            }
            // Leading half of a Windows line ending.
            '\r' if !quoted => {}
            character => field.push(character),
        }
    }

    // A file whose last line has no line ending still holds a record.
    if !field.is_empty() || !record.is_empty() {
        record.push(field);
        records.push(record);
    }

    records
}

/// True when a record holds nothing, as a blank line does.
pub fn is_blank(record: &[String]) -> bool {
    record.iter().all(|field| field.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_plain_records() {
        assert_eq!(
            parse("Name,Quantity\nSol Ring,2\n"),
            [["Name", "Quantity"], ["Sol Ring", "2"]]
        );
    }

    #[test]
    fn keeps_commas_inside_quoted_fields() {
        assert_eq!(
            parse("\"Chandra, Torch of Defiance\",1\n"),
            [["Chandra, Torch of Defiance", "1"]]
        );
    }

    #[test]
    fn reads_doubled_quotes_as_one_quote() {
        assert_eq!(parse("\"say \"\"hi\"\"\",1\n"), [["say \"hi\"", "1"]]);
    }

    #[test]
    fn reads_line_endings_and_embedded_newlines() {
        assert_eq!(
            parse("a,b\r\n\"one\ntwo\",c\r\n"),
            [["a", "b"], ["one\ntwo", "c"]]
        );
    }

    #[test]
    fn reads_a_last_line_without_line_ending() {
        assert_eq!(parse("a,b\nc,d"), [["a", "b"], ["c", "d"]]);
    }

    #[test]
    fn reports_blank_records() {
        let records = parse("a,b\n\nc,d\n");

        assert_eq!(records.len(), 3);
        assert!(is_blank(&records[1]));
        assert!(!is_blank(&records[0]));
    }
}
