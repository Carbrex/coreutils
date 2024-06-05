// This file is part of the uutils coreutils package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore (ToDO) extendedbigdecimal numberparse
use std::io::{stdout, BufRead, ErrorKind, Write};

use clap::{crate_version, Arg, ArgAction, Command};
use num_traits::{Float, ToPrimitive, Zero};

use uucore::error::{FromIo, UResult};
use uucore::format::{num_format, Format};
use uucore::{format_usage, help_about, help_usage};

mod error;
mod extendedbigdecimal;
mod number;
mod numberparse;
use crate::error::SeqError;
use f128::f128;
use numberparse::ParseNumberError;

const ABOUT: &str = help_about!("seq.md");
const USAGE: &str = help_usage!("seq.md");

const OPT_SEPARATOR: &str = "separator";
const OPT_TERMINATOR: &str = "terminator";
const OPT_EQUAL_WIDTH: &str = "equal-width";
const OPT_FORMAT: &str = "format";

const ARG_NUMBERS: &str = "numbers";

#[derive(Clone)]
struct SeqOptions<'a> {
    separator: String,
    terminator: String,
    equal_width: bool,
    format: Option<&'a str>,
}

/// A range of floats.
///
/// The elements are (first, increment, last).
type RangeFloat = (f128, f128, f128);

fn num_integral_digits(num: f128) -> usize {
    let mut num = f128::abs(num);
    let mut digits = 0;
    while num >= f128::ONE {
        num = num / f128::new(10);
        digits += 1;
    }
    digits
}

fn num_fractional_digits(num: f128) -> usize {
    let s = format!("{}", num);
    if let Some(pos) = s.find('.') {
        s[pos+1..].len()
    } else {
        0
    }
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let numbers_option = matches.get_many::<String>(ARG_NUMBERS);

    if numbers_option.is_none() {
        return Err(SeqError::NoArguments.into());
    }

    let numbers = numbers_option.unwrap().collect::<Vec<_>>();

    let options = SeqOptions {
        separator: matches
            .get_one::<String>(OPT_SEPARATOR)
            .map(|s| s.as_str())
            .unwrap_or("\n")
            .to_string(),
        terminator: matches
            .get_one::<String>(OPT_TERMINATOR)
            .map(|s| s.as_str())
            .unwrap_or("\n")
            .to_string(),
        equal_width: matches.get_flag(OPT_EQUAL_WIDTH),
        format: matches.get_one::<String>(OPT_FORMAT).map(|s| s.as_str()),
    };

    let first = if numbers.len() > 1 {
        match f128::parse(numbers[0]) {
            Ok(num) => num,
            Err(_e) => {
                return Err(
                    SeqError::ParseError(numbers[0].to_string(), ParseNumberError::Float).into(),
                )
            }
        }
    } else {
        f128::ONE
    };
    if first.is_infinite() {
        return Err(SeqError::ParseError(numbers[0].to_string(), ParseNumberError::Float).into());
    }
    let increment = if numbers.len() > 2 {
        match f128::parse(numbers[1]) {
            Ok(num) => num,
            Err(_e) => {
                return Err(
                    SeqError::ParseError(numbers[1].to_string(), ParseNumberError::Float).into(),
                )
            }
        }
    } else {
        f128::ONE
    };
    if increment.is_infinite() {
        return Err(SeqError::ParseError(numbers[1].to_string(), ParseNumberError::Float).into());
    }
    if increment.is_zero() {
        return Err(SeqError::ZeroIncrement(numbers[1].to_string()).into());
    }
    let last = {
        let n: usize = numbers.len();
        match f128::parse(numbers[n - 1]) {
            Ok(num) => num,
            Err(_e) => {
                return Err(SeqError::ParseError(
                    numbers[n - 1].to_string(),
                    ParseNumberError::Float,
                )
                .into())
            }
        }
    };
    if last.is_infinite() {
        return Err(SeqError::ParseError(numbers[2].to_string(), ParseNumberError::Float).into());
    }

    let padding = num_integral_digits(first)
        .max(num_integral_digits(increment))
        .max(num_integral_digits(last));
    let largest_dec = num_fractional_digits(first).max(num_fractional_digits(increment));
    println!("padding: {}", padding);
    println!("largest_dec: {}", largest_dec);
    println!("first: {}", first);
    println!("increment: {:?}", increment);
    println!("last: {}", last);
    let format = match options.format {
        Some(f) => {
            let f = Format::<num_format::Float>::parse(f)?;
            Some(f)
        }
        None => None,
    };
    let result = print_seq(
        (first, increment, last),
        largest_dec,
        &options.separator,
        &options.terminator,
        options.equal_width,
        padding,
        &format,
    );
    match result {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e.map_err_context(|| "write error".into())),
    }
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .trailing_var_arg(true)
        .allow_negative_numbers(true)
        .infer_long_args(true)
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .arg(
            Arg::new(OPT_SEPARATOR)
                .short('s')
                .long("separator")
                .help("Separator character (defaults to \\n)"),
        )
        .arg(
            Arg::new(OPT_TERMINATOR)
                .short('t')
                .long("terminator")
                .help("Terminator character (defaults to \\n)"),
        )
        .arg(
            Arg::new(OPT_EQUAL_WIDTH)
                .short('w')
                .long("equal-width")
                .help("Equalize widths of all numbers by padding with zeros")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(OPT_FORMAT)
                .short('f')
                .long(OPT_FORMAT)
                .help("use printf style floating-point FORMAT"),
        )
        .arg(
            Arg::new(ARG_NUMBERS)
                .action(ArgAction::Append)
                .num_args(1..=3),
        )
}

fn done_printing<T: Zero + PartialOrd>(next: &T, increment: &T, last: &T) -> bool {
    if increment >= &T::zero() {
        next > last
    } else {
        next < last
    }
}

fn write_value_float(
    writer: &mut impl Write,
    value: &f128,
    width: usize,
    precision: usize,
) -> std::io::Result<()> {
    println!("{:?}", value);
    write!(writer, "{:0width$.precision$}", value, width = width, precision = precision)
}

/// Write a floating point value to a writer.
// fn write_value_float(
//     writer: &mut impl Write,
//     value: &f128,
//     width: usize,
//     precision: usize,
// ) -> std::io::Result<()> {
//     // let value_as_str = if *value == f128::INFINITY || *value == f128::NEG_INFINITY {
//     //     format!("{value:>width$.precision$}")
//     // } else {
//     //     format!("{value:>0width$.precision$}")
//     // };
//     // write!(writer, "{value_as_str}")
//     // let value_as_f64_option = value.to_f64();
//     // match value_as_f64_option {
//     //     Some(value_as_f64) => {
//     //         write!(
//     //             writer,
//     //             "{:0width$.precision$}",
//     //             value_as_f64,
//     //             width = width,
//     //             precision = precision
//     //         )
//     //     },
//     //     None => {
//     //         write!(writer, "Error: value could not be converted to f64")
//     //     }
//     // }
//     write!(
//         writer,
//         "{:0width$.precision$}",
//         value,
//         width = width,
//         precision = precision
    
//     )
// }


/// Floating point based code path
fn print_seq(
    range: RangeFloat,
    largest_dec: usize,
    separator: &str,
    terminator: &str,
    pad: bool,
    padding: usize,
    format: &Option<Format<num_format::Float>>,
) -> std::io::Result<()> {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let (first, increment, last) = range;
    let mut value = first;
    let padding = if pad {
        padding + if largest_dec > 0 { largest_dec + 1 } else { 0 }
    } else {
        0
    };
    let mut is_first_iteration = true;
    while !done_printing(&value, &increment, &last) {
        if !is_first_iteration {
            write!(stdout, "{separator}")?;
        }
        // match &format {
            //     Some(f) => {
        //         let float = match &value {
            //             // ExtendedBigDecimal::BigDecimal(bd) => bd.to_f64().unwrap(),
            //             // ExtendedBigDecimal::Infinity => f64::INFINITY,
            //             // ExtendedBigDecimal::MinusInfinity => f64::NEG_INFINITY,
        //             // ExtendedBigDecimal::MinusZero => -0.0,
        //             // ExtendedBigDecimal::Nan => f64::NAN,
        //             f if f.is_infinite() && f.is_sign_positive() => f64::INFINITY,
        //             f if f.is_infinite() && f.is_sign_negative() => f64::NEG_INFINITY,
        //             _ => value.to_f64().unwrap(),
        //         };
        //         f.fmt(&mut stdout, float)?;
        //     }
        //     None => write_value_float(&mut stdout, &value, padding, largest_dec)?,
        // }
        // println!("value: {}", value);
        write_value_float(&mut stdout, &value, padding, largest_dec)?;
        break;
        // TODO Implement augmenting addition.
        value = value + increment.clone();
        is_first_iteration = false;
    }
    if !is_first_iteration {
        write!(stdout, "{terminator}")?;
    }
    stdout.flush()?;
    Ok(())
}
