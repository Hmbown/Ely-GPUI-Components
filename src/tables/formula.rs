use std::sync::LazyLock;

use gpui::SharedString;
use regex::{Captures, Regex};

use crate::{
    forms::evaluate,
    typography::format::{self, Separators},
};

static RANGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b([A-Z]+)\(\s*([A-Z]+[0-9]+)\s*:\s*([A-Z]+[0-9]+)\s*\)")
        .expect("a range pattern")
});
static CALL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([A-Za-z]+)\(").expect("a call pattern"));
static REFERENCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([A-Z]+)([0-9]+)\b").expect("a reference pattern"));

/// How deep formulas may lean on each other before a loop is assumed.
const DEPTH: usize = 64;

/// A column's letters: A to Z, then AA.
pub(crate) fn letters(mut col: usize) -> String {
    let mut out = Vec::new();
    loop {
        out.push(b'A' + (col % 26) as u8);
        if col < 26 {
            break;
        }
        col = col / 26 - 1;
    }
    out.reverse();
    String::from_utf8(out).expect("letters are ASCII")
}

/// A cell's name, as B3 for row 2 and column 1, counted from zero.
pub(crate) fn name(row: usize, col: usize) -> String {
    format!("{}{}", letters(col), row + 1)
}

/// The row and column an A1 name points to, counted from zero.
pub(crate) fn place(text: &str) -> Option<(usize, usize)> {
    let caps = REFERENCE
        .captures(text)
        .filter(|caps| caps.get(0).is_some_and(|all| all.as_str() == text))?;
    let col = caps[1].bytes().fold(0usize, |col, letter| {
        col * 26 + usize::from(letter - b'A' + 1)
    }) - 1;
    let row = caps[2].parse::<usize>().ok()?.checked_sub(1)?;
    Some((row, col))
}

/// Every cell name in a rectangle, row by row.
fn span(from: &str, to: &str) -> Result<Vec<String>, String> {
    let ((r0, c0), (r1, c1)) = (
        place(from).ok_or("a bad cell")?,
        place(to).ok_or("a bad cell")?,
    );
    let (rows, cols) = (r0.min(r1)..=r0.max(r1), c0.min(c1)..=c0.max(c1));
    Ok(rows
        .flat_map(|row| cols.clone().map(move |col| name(row, col)))
        .collect())
}

/// A formula with its ranges written out and its functions in the evaluator's words.
fn expand(source: &str) -> Result<String, String> {
    let mut failure = None;
    let written = RANGE.replace_all(source, |caps: &Captures| {
        let cells = match span(&caps[2], &caps[3]) {
            Ok(cells) => cells,
            Err(error) => {
                failure = Some(error);
                return String::new();
            }
        };
        let (sum, count) = (cells.join("+"), cells.len());
        match caps[1].to_uppercase().as_str() {
            "SUM" => format!("({sum})"),
            "AVERAGE" | "AVG" => format!("(({sum})/{count})"),
            "MIN" => format!("min({})", cells.join(",")),
            "MAX" => format!("max({})", cells.join(",")),
            "COUNT" => count.to_string(),
            other => {
                failure = Some(format!("{other} takes no range"));
                String::new()
            }
        }
    });
    if let Some(error) = failure {
        return Err(error);
    }
    Ok(CALL
        .replace_all(&written, |caps: &Captures| {
            format!("{}(", caps[1].to_lowercase())
        })
        .into_owned())
}

/// A cell's number: its own, or its formula's result; text and blanks have none.
pub(crate) fn value(
    cells: &dyn Fn(usize, usize) -> SharedString,
    row: usize,
    col: usize,
    depth: usize,
) -> Result<Option<f64>, String> {
    let raw = cells(row, col);
    match raw.strip_prefix('=') {
        Some(source) => formula(cells, source, depth).map(Some),
        None => Ok(raw.trim().parse::<f64>().ok()),
    }
}

fn formula(
    cells: &dyn Fn(usize, usize) -> SharedString,
    source: &str,
    depth: usize,
) -> Result<f64, String> {
    if depth > DEPTH {
        return Err("a loop".into());
    }
    let expanded = expand(source)?;
    let mut variables: Vec<(SharedString, f64)> = Vec::new();
    for caps in REFERENCE.captures_iter(&expanded) {
        let reference = caps[0].to_string();
        if variables
            .iter()
            .any(|(known, _)| known.as_ref() == reference)
        {
            continue;
        }
        let (row, col) = place(&reference).ok_or_else(|| format!("{reference} is not a cell"))?;
        let number = value(cells, row, col, depth + 1)?.unwrap_or(0.0);
        variables.push((reference.into(), number));
    }
    evaluate(&expanded, &variables)
}

/// What a cell shows: a formula's result, or #ERR when it fails; other text as written.
pub(crate) fn shown(
    cells: &dyn Fn(usize, usize) -> SharedString,
    row: usize,
    col: usize,
) -> SharedString {
    let raw = cells(row, col);
    if !raw.starts_with('=') {
        return raw;
    }
    match value(cells, row, col, 0) {
        Ok(Some(number)) => format::number(
            number,
            if number.fract() == 0.0 { 0 } else { 2 },
            Separators::EN,
        )
        .into(),
        Ok(None) => SharedString::default(),
        Err(error) => {
            log::debug!("spreadsheet: {} failed: {error}", name(row, col));
            "#ERR".into()
        }
    }
}

/// A formula with each reference moved by `rows` and `cols`, as a fill carries it.
pub(crate) fn shifted(source: &str, rows: isize, cols: isize) -> String {
    REFERENCE
        .replace_all(source, |caps: &Captures| match place(&caps[0]) {
            Some((row, col)) => {
                let (row, col) = (row as isize + rows, col as isize + cols);
                if row < 0 || col < 0 {
                    "#REF".to_string()
                } else {
                    name(row as usize, col as usize)
                }
            }
            None => caps[0].to_string(),
        })
        .into_owned()
}

/// What a fill writes into `count` cells past a run, down or `across`: numbers that step evenly keep stepping; a formula shifts; anything else repeats in turn.
pub(crate) fn filled(run: &[SharedString], count: usize, across: bool) -> Vec<SharedString> {
    let numbers: Option<Vec<f64>> = run
        .iter()
        .map(|text| text.trim().parse::<f64>().ok())
        .collect();
    if let Some(numbers) = numbers.filter(|numbers| numbers.len() >= 2) {
        let step = numbers[1] - numbers[0];
        let even = numbers
            .windows(2)
            .all(|pair| (pair[1] - pair[0] - step).abs() < 1e-9);
        if even {
            let last = numbers[numbers.len() - 1];
            return (1..=count)
                .map(|ix| format!("{}", last + step * ix as f64).into())
                .collect();
        }
    }
    (0..count)
        .map(|ix| {
            let source = &run[ix % run.len()];
            let distance = (run.len() + ix - ix % run.len()) as isize;
            match source.strip_prefix('=') {
                Some(formula) if across => format!("={}", shifted(formula, 0, distance)).into(),
                Some(formula) => format!("={}", shifted(formula, distance, 0)).into(),
                None => source.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet(rows: &[&[&'static str]]) -> impl Fn(usize, usize) -> SharedString {
        let rows: Vec<Vec<SharedString>> = rows
            .iter()
            .map(|row| row.iter().map(|cell| SharedString::from(*cell)).collect())
            .collect();
        move |row, col| {
            rows.get(row)
                .and_then(|cells| cells.get(col))
                .cloned()
                .unwrap_or_default()
        }
    }

    #[test]
    fn names_and_places_meet() {
        assert_eq!(letters(0), "A");
        assert_eq!(letters(25), "Z");
        assert_eq!(letters(26), "AA");
        assert_eq!(name(2, 1), "B3");
        assert_eq!(place("B3"), Some((2, 1)));
        assert_eq!(place("AA10"), Some((9, 26)));
        assert_eq!(place("B0"), None);
    }

    #[test]
    fn formulas_add_ranges_and_lean_on_each_other() {
        let cells = sheet(&[
            &["4", "=A1*2", "=SUM(A1:B1)"],
            &["6", "=average(A1:A2)", "=MAX(A1:B2)"],
            &["x", "=A3+1", "=C3"],
        ]);
        assert_eq!(shown(&cells, 0, 1), "8");
        assert_eq!(shown(&cells, 0, 2), "12");
        assert_eq!(shown(&cells, 1, 1), "5");
        assert_eq!(shown(&cells, 1, 2), "8");
        assert_eq!(shown(&cells, 2, 1), "1", "text counts as nothing");
        assert_eq!(
            shown(&cells, 2, 2),
            "#ERR",
            "a cell leaning on itself is a loop"
        );
        let grouped = sheet(&[&["4", "6", "=SUM(A1:B1)*2"]]);
        assert_eq!(
            shown(&grouped, 0, 2),
            "20",
            "a range sums before it multiplies"
        );
    }

    #[test]
    fn fills_step_numbers_shift_formulas_and_repeat_text() {
        let texts = |list: &[&'static str]| {
            list.iter()
                .map(|text| SharedString::from(*text))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            filled(&texts(&["2", "4"]), 3, false),
            texts(&["6", "8", "10"])
        );
        assert_eq!(
            filled(&texts(&["=A1*2"]), 2, false),
            texts(&["=A2*2", "=A3*2"])
        );
        assert_eq!(filled(&texts(&["=A1*2"]), 1, true), texts(&["=B1*2"]));
        assert_eq!(
            filled(&texts(&["Mon", "Tue"]), 3, false),
            texts(&["Mon", "Tue", "Mon"])
        );
        assert_eq!(shifted("B2-A1", -1, 0), "B1-#REF");
    }
}
