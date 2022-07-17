use clap::{App, Arg};
use std::error::Error;

use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    show_count: bool,
}

pub fn run(config: &Config) -> MyResult<()> {
    let reader = get_reader(&config.in_file)
        .map_err(|e| format!("{}: {}: {}", "unic", config.in_file, e))?;
    let mut writer: Box<dyn Write> = get_writer(&config.out_file)?;
    process_read_write(reader, &mut writer, &config)?;
    Ok(())
}

fn process_read_write(
    mut reader: impl BufRead,
    writer: &mut impl Write,
    config: &Config,
) -> MyResult<()> {
    let mut buf = String::new();
    let mut prev: String = String::new();
    let mut running_count = 0 as usize;

    let mut write_line = |count: usize, old_line: &String| -> MyResult<()> {
        let line = if config.show_count {
            format!("{:>4} {}", count, old_line)
        } else {
            format!("{}", old_line)
        };
        writer.write_all(line.as_bytes())?;
        Ok(())
    };

    loop {
        let bytes_read = reader.read_line(&mut buf)?;
        if bytes_read == 0 {
            break;
        }

        /*
        This below behaviour of skipping line end whitespaces before comparing makes sense logically
        but is different from how bsd uniq works.

        This can be proved by running uniq on the files created as below:
        printf "a\na" | uniq => outputs 'a\n'
        printf "a\r\na\n" | uniq => outputs 'a\na\n'
        printf "a\na\n" | uniq => outputs 'a\n\
         */
        if prev.trim_end() != buf.trim_end() {
            if running_count > 0 {
                write_line(running_count, &prev)?;
                running_count = 0;
            }
            prev = buf.clone();
        }
        running_count += 1;
        buf.clear();
    }
    if running_count > 0 {
        write_line(running_count, &prev)?;
    }
    Ok(())
}

/// Parse the config out of the provided command line arguments
pub fn parse_config(cmd_args: Vec<String>) -> MyResult<Config> {
    let matches = App::new("unic")
        .version("1.0.0")
        .author("sanjayts")
        .arg(
            Arg::new("in_file")
                .value_name("INPUT")
                .multiple_values(false)
                .default_value("-"),
        )
        .arg(
            Arg::new("out_file")
                .value_name("OUTPUT")
                .multiple_values(false),
        )
        .arg(
            Arg::new("show_count")
                .short('c')
                .long("count")
                .takes_value(false)
                .help("Prefix lines by the number of occurrences"),
        )
        .get_matches_from(cmd_args);

    // FIXME Is there a way to avoid cloning here?
    let in_file = matches.get_one::<String>("in_file").unwrap().to_string();
    let out_file = matches.get_one::<String>("out_file").cloned();
    let show_count = matches.is_present("show_count");

    let config = Config {
        in_file,
        out_file,
        show_count,
    };
    Ok(config)
}

fn get_reader(file_name: &str) -> MyResult<Box<dyn BufRead>> {
    match file_name {
        "-" => Ok(Box::new(BufReader::new(stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(file_name)?))),
    }
}

fn get_writer(file_name: &Option<String>) -> MyResult<Box<dyn Write>> {
    match file_name {
        Some(file_name) => Ok(Box::new(BufWriter::new(File::create(file_name)?))),
        None => Ok(Box::new(BufWriter::new(stdout()))),
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_config, process_read_write, Config};
    use std::io::{Cursor, Write};

    #[test]
    fn test_parse_config_all_provided() {
        // It's important to pass in the first arg as some string otherwise clap considers our
        // -c flag as the program name and only picks up the in_file and out_file from the args!
        let args_1 = vec!["my-executable", "-c", "infile", "outfile"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let res_1 = parse_config(args_1);

        assert!(res_1.is_ok());
        assert_eq!(
            res_1.unwrap(),
            Config {
                in_file: "infile".to_string(),
                out_file: Some("outfile".to_string()),
                show_count: true,
            }
        )
    }

    #[test]
    fn test_parse_config_no_args() {
        let args_1 = vec!["my-executable".to_string()];
        let res_1 = parse_config(args_1);

        assert!(res_1.is_ok());
        assert_eq!(
            res_1.unwrap(),
            Config {
                in_file: "-".to_string(),
                out_file: None,
                show_count: false,
            }
        )
    }

    #[ignore]
    #[test]
    // FIXME figure out how we can disallow flags after positional args the way BSD uniq does it
    fn test_parse_no_flags_after_file() {
        let args_1 = vec!["my-executable", "infile", "-c"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let res_1 = parse_config(args_1);

        assert!(res_1.is_err());
    }

    #[test]
    fn test_read_write_default() {
        // One cool trick I learnt online is that we can use a vec to test out our simple read
        // writes! https://stackoverflow.com/a/50732452/193906
        let config = Config {
            in_file: "".to_string(),
            out_file: None,
            show_count: false,
        };
        let mut reader: Vec<u8> = vec![];
        let mut writer: Vec<u8> = vec![];
        reader.write_all("a\na\nb\nb\n\n\n".as_bytes()).unwrap();

        let res = process_read_write(Cursor::new(reader), &mut writer, &config);
        assert!(res.is_ok());

        let actual = String::from_utf8(writer).unwrap();
        assert_eq!(actual, "a\nb\n\n")
    }

    #[test]
    fn test_read_write_with_count() {
        let config = Config {
            in_file: "".to_string(),
            out_file: None,
            show_count: true,
        };
        let mut reader: Vec<u8> = vec![];
        let mut writer: Vec<u8> = vec![];
        reader.write_all("a\na\nb\nb\n\n\n".as_bytes()).unwrap();

        let res = process_read_write(Cursor::new(reader), &mut writer, &config);
        assert!(res.is_ok());

        let actual = String::from_utf8(writer).unwrap();
        assert_eq!(actual, "   2 a\n   2 b\n   2 \n")
    }

}
