use std::fmt;

use lalrpop_util::lalrpop_mod;

mod lex;
lalrpop_mod!(csv);

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("parse error occured {0}")]
    ParseError(#[from] lalrpop_util::ParseError<usize, lex::Token<String>, String>),
}

#[derive(Debug)]
pub struct Csv {
    header: Option<(Row, &'static str)>,
    rows: Vec<(Row, Option<&'static str>)>,
}

impl Csv {
    fn new(v: Vec<(Row, &'static str)>, r: Row, eol: Option<&'static str>) -> Self {
        let mut rows = v.into_iter().map(|(r, l)| (r, Some(l))).collect::<Vec<_>>();
        rows.push((r, eol));
        Self { header: None, rows }
    }

    fn new_with_header(
        h: (Row, &'static str),
        v: Vec<(Row, &'static str)>,
        r: Row,
        eol: Option<&'static str>) -> Self {

        let mut rows = v.into_iter().map(|(r, l)| (r, Some(l))).collect::<Vec<_>>();
        rows.push((r, eol));
        Self { header: Some(h), rows }
    }

    pub fn parse<'input>(input: &'input str, header: bool) -> Result<Csv, ParseError> {
        if header {
            Self::parse_with_header(input)
        } else {
            Self::parse_without_header(input)
        }
    }

    pub fn parse_without_header<'input>(input: &'input str) -> Result<Csv, ParseError> {
        let lexer = lex::Lexer::new(&input);
        let result = csv::CsvParser::new()
            .parse(&input, lexer).map_err(|e|e.map_token(lex::Token::to_owned))?;
        Ok(result)
    }

    pub fn parse_with_header<'input>(input: &'input str) -> Result<Csv, ParseError> {
        let lexer = lex::Lexer::new(&input);
        let result = csv::CsvWithHeaderParser::new()
            .parse(&input, lexer).map_err(|e|e.map_token(lex::Token::to_owned))?;
        Ok(result)
    }

    /*
    pub fn add_row(&mut self) -> usize {
        let row = Row {
            cells: vec![],
        };
        self.rows.push((row, Some("\r\n")));
        let len = self.rows.len();
        len - 1
    }
    */

    pub fn header(&self, col: usize) -> Option<&str> {
        self.header.as_ref().and_then(|(h, _)| h.cells.get(col)).map(Cell::val)
    }

    pub fn rows(&self) -> usize {
        self.rows.len()
    }

    pub fn max_cols(&self) -> usize {
        self.rows.iter().map(|(r, _)| r.cells.len()).max().unwrap_or(0)
    }

    pub fn vals(&self, row: usize) -> impl Iterator<Item=&str> + '_ {
        self.rows.get(row).map(|(r, _)| r.cells.iter().map(Cell::val)).into_iter().flatten()
    }

    pub fn set_val<S:ToString>(&mut self, row: usize, col: usize, val: S) -> bool {
        let maybe_cell = self.rows.get_mut(row).and_then(|(r, _)| r.cells.get_mut(col));
        if let Some(cell) = maybe_cell {
            cell.set_val(val.to_string());
            true
        } else {
            false
        }
    }

    pub fn cols(&self, col: usize) -> impl Iterator<Item=&str> + '_ {
        self.rows.iter().filter_map(move |(r, _)| r.cells.get(col)).map(Cell::val)
    }
}

impl fmt::Display for Csv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((header, eol)) = &self.header {
            write!(f, "{}{}", header, eol)?;
        };

        for (row, eol) in &self.rows {
            if let Some(eol) = eol {
                write!(f, "{}{}", row, eol)?;
            } else {
                write!(f, "{}", row)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Row {
    cells: Vec<Cell>,
}

impl Row {
    fn new(v: Vec<Cell>, r: Cell) -> Self {
        let mut cells = v;
        cells.push(r);
        Self { cells }
    }

    /*
    pub fn add_col<S: ToString>(&mut self, val: S) -> &mut Cell {
        let val = val.to_string();
        let cell = Cell {
            quoted: val.contains(&[',', '\r', '"', '\n'][..]),
            val,
        };
        self.cells.push(cell);
        let len = self.cells.len();
        &mut self.cells[len - 1]
    }
    */
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.cells.iter();
        if let Some(cell) = iter.next() {
            write!(f, "{}", cell)?;
        } else {
            return Ok(())
        }
        while let Some(cell) = iter.next() {
            write!(f, ",{}", cell)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Cell {
    val: String,
    quoted: bool,
}

impl Cell {
    fn new(quoted: bool, v: &[&str]) -> Self {
        Self {
            quoted,
            val: v.into_iter().cloned().collect(),
        }
    }

    fn val(&self) -> &str {
        &self.val
    }

    fn set_val<S: ToString>(&mut self, val: S) {
        let val = val.to_string();
        if !self.quoted && val.contains(&[',', '\r', '"', '\n'][..]) {
            self.quoted = true
        }
        self.val = val;
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.quoted {
            write!(f, "\"{}\"", self.val.replace('"', "\"\""))
        } else {
            write!(f, "{}", self.val)
        }
    }
}
