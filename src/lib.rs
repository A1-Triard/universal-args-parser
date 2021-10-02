#![no_std]

#[cfg(test)]
extern crate alloc;

use core::concat;
use core::fmt::{self, Display, Formatter, Write};
use core::option::Option;

#[allow(non_snake_case)]
pub struct OptArg<'s> {
    pub optional: bool,
    /// This string should be in normalized form and not contains incomplete graphemes.
    /// Also, all graphemes should have unit length.
    /// Max chars count = 256;
    pub NAME: &'s str,
}

pub struct Opt<'s, O> {
    pub key: O,
    /// This string should be in normalized form and not contains incomplete graphemes.
    /// Also, all graphemes should have unit length.
    /// Max chars count = 256;
    pub long: &'s str,
    /// A unicode grapheme consists from the char should exists and has unit length.
    pub short: Option<char>,
    pub arg: Option<OptArg<'s>>,
    pub doc: &'s str,
}

impl<'s, O> Opt<'s, O> {
    fn long_length(&self) -> usize {
        self.long.chars().take(256).count() +
        self.arg.as_ref().map_or(0, |x| 1 + x.NAME.chars().take(256).count())
    }

    fn print_long(&self, long_opt: &str, f: &mut Formatter) -> fmt::Result {
        write!(f, "{lo}{long}", lo=long_opt, long=self.long)?;
        if let Some(arg) = self.arg.as_ref() {
            write!(f, "={name}", name=arg.NAME)?;
        }
        Ok(())
    }

    fn print_short(&self, short_opt: &str, f: &mut Formatter) -> fmt::Result {
        if let Some(short) = self.short {
            write!(f, "{so}{short}", so=short_opt, short=short)?;
        } else {
            tab(short_opt.chars().count(), f)?;
            f.write_char(' ')?;
        }
        Ok(())
    }

    fn print_comma_between_short_and_long(&self, f: &mut Formatter) -> Result<bool, fmt::Error> {
        if self.short.is_some() && !self.long.is_empty() {
            f.write_str(", ")?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
#[allow(non_snake_case)]
pub struct Args<'o, 's, O> {
    pub opts: &'o [Opt<'s, O>],
    pub prog: &'s str,
    pub OPTION: &'s str,
    pub Usage: &'s str,
    pub Mandatory_arguments_to_long_options_are_mandatory_for_short_options_too_: &'s str,
    pub newline: &'s str,
    pub Doc_: &'s str,
    /// This string should be in normalized form and not contains incomplete graphemes.
    /// Also, all graphemes should have unit length.
    /// Max chars count = 256;
    pub short_opt: &'s str,
    /// This string should be in normalized form and not contains incomplete graphemes.
    /// Also, all graphemes should have unit length.
    /// Max chars count = 256;
    pub long_opt: &'s str,
}

pub struct ArgsUsage<'a, 'o, 's, O>(&'a Args<'o, 's, O>);

fn tab(n: usize, f: &mut Formatter) -> fmt::Result {
    for _ in 0 .. n {
        f.write_char(' ')?;
    }
    Ok(())
}

impl<'a, 'o, 's, O> Display for ArgsUsage<'a, 'o, 's, O> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{usage}: {prog}", usage=self.0.Usage, prog=self.0.prog)?;
        if !self.0.opts.is_empty() {
            write!(f, " [{option}]...", option=self.0.OPTION)?;
        }
        if !self.0.Doc_.is_empty() {
            write!(f, "{nl}{doc}", nl=self.0.newline, doc=self.0.Doc_)?;
        }
        f.write_str(self.0.newline)?;
        if !self.0.opts.is_empty() {
            let long_len = self.0.opts.iter().map(|x| x.long_length()).max().unwrap();
            f.write_str(self.0.newline)?;
            if self.0.opts.iter().filter_map(|x| x.arg.as_ref()).any(|x| !x.optional) {
                f.write_str(self.0.Mandatory_arguments_to_long_options_are_mandatory_for_short_options_too_)?;
                f.write_str(self.0.newline)?;
                f.write_str(self.0.newline)?;
            }
            for opt in self.0.opts {
                f.write_str("  ")?;
                opt.print_short(self.0.short_opt, f)?;
                let comma_printed = opt.print_comma_between_short_and_long(f)?;
                if !comma_printed && (!opt.long.is_empty() || !opt.doc.is_empty()) {
                    f.write_str("  ")?;
                }
                opt.print_long(self.0.long_opt, f)?;
                if !opt.doc.is_empty() {
                    tab(long_len - opt.long_length(), f)?;
                    write!(f, " {doc}", doc=opt.doc)?;
                }
                f.write_str(self.0.newline)?;
            }
        }
        Ok(())
    }
}

impl<'o, 's, O> Args<'o, 's, O> {
    pub fn usage<'a>(&'a self) -> ArgsUsage<'a, 'o, 's, O> { ArgsUsage(self) }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use alloc::string::ToString;

    enum GzipOptions {
        Ascii,
        Stdout,
        Rsyncable,
    }

    #[test]
    fn it_works() {
        let args = Args {
            short_opt: "-",
            long_opt: "--",
            Usage: "Usage",
            Mandatory_arguments_to_long_options_are_mandatory_for_short_options_too_:
                "Mandatory arguments to long options are mandatory for short options too.",
            newline: "\n",
            prog: "gzip",
            OPTION: "OPTION",
            Doc_: "Compress or uncompress FILEs (by default, compress FILEs in-place).",
            opts: &[
                Opt {
                    key: GzipOptions::Ascii,
                    long: "ascii",
                    short: Some('a'),
                    arg: None,
                    doc: "ascii text; convert end-of-line using local conventions",
                },
                Opt {
                    key: GzipOptions::Stdout,
                    long: "stdout",
                    short: Some('c'),
                    arg: None,
                    doc: "write on standard output, keep original files unchanged",
                },
                Opt {
                    key: GzipOptions::Rsyncable,
                    long: "rsyncable",
                    short: None,
                    arg: None,
                    doc: "make rsync-friendly archive",
                },
                Opt {
                    key: GzipOptions::Rsyncable,
                    long: "suffix",
                    short: Some('S'),
                    arg: Some(OptArg {
                        NAME: "SUF",
                        optional: false,
                    }),
                    doc: "use suffix SUF on compressed files",
                },
            ]
        };
        assert_eq!(args.usage().to_string(), concat!(
            "Usage: gzip [OPTION]...\n",
            "Compress or uncompress FILEs (by default, compress FILEs in-place).\n",
            "\n",
            "Mandatory arguments to long options are mandatory for short options too.\n",
            "\n",
            "  -a, --ascii      ascii text; convert end-of-line using local conventions\n",
            "  -c, --stdout     write on standard output, keep original files unchanged\n",
            "      --rsyncable  make rsync-friendly archive\n",
            "  -S, --suffix=SUF use suffix SUF on compressed files\n",
        ));
    }
}
