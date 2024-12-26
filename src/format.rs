use core::f64;

use crate::interpreter::unit_prefix::UnitPrefix;

pub(crate) fn as_bin(num: u64) -> String {
    let bin = format!("{:b}", num);
    let pad = if bin.len() % 8 != 0 {
        8 - bin.len() % 8
    } else {
        0
    };

    // Insert spaces every 8 characters
    let bin = bin
        .chars()
        .rev()
        .enumerate()
        .fold(String::new(), |mut acc, (i, c)| {
            if i % 8 == 0 && i != 0 {
                acc.push(' ');
            }
            acc.push(c);
            acc
        })
        .chars()
        .rev()
        .collect::<String>();

    // Pad with 0 to multiple of 8 length
    let mut padded_bin = "0".repeat(pad);
    padded_bin.push_str(&bin);

    padded_bin
}

pub(crate) fn as_dec_size(num: u64) -> String {
    let prefix = UnitPrefix::dec_from_num(num);

    let fnum = num as f64 / u64::from(prefix) as f64;
    let digits = (num % 1000 != 0) as usize;
    format!("{:.1$} {2}B", fnum, digits, prefix)
}

pub(crate) fn as_bin_size(num: u64) -> String {
    let prefix = UnitPrefix::bin_from_num(num);

    let fnum = num as f64 / u64::from(prefix) as f64;
    let digits = (num % 1024 != 0) as usize;
    format!("{:.1$} {2}B", fnum, digits, prefix)
}
