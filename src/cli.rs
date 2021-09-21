use clap::{App, Arg, SubCommand};
use std::collections::HashSet;

#[derive(Default, Debug)]
pub struct Options {
    pub show_local_fs: bool,
    pub show_all_fs: bool,
    pub listed_fs: HashSet<String>,
    pub excluded_fs: HashSet<String>,
    pub human_readable: bool,
    pub print_grand_total: bool,
    pub field_list: Vec<String>,
    pub human_readable_1024: bool, // true => show size in powers of 1024, false => powers of 1000
    pub inodes: bool,
    pub show_fs_type: bool,
    pub output_all_fields: bool,
}

impl Options {
    pub fn new() -> Options {
        Options {
            show_local_fs: true,
            show_all_fs: false,
            human_readable: true,
            ..Default::default()
        }
    }
}

pub fn parse_args() -> Options {
    let mut options = Options::new();
    let matches = App::new("df.rust")
        .version("0.1.0")
        .author("Hedonihilist")
        .about("df written in Rust")
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Show all file systems"),
        )
        .arg(
            // core features first, ignore this option for now
            Arg::with_name("block_size")
                .short("B")
                .long("block-size")
                .value_name("SIZE")
                .help("scale sizes by size before printing them")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("human_readable_1024")
                .short("h")
                .long("human-readable")
                .help("print sizes in powers of 1024"),
        )
        .arg(
            Arg::with_name("human_readable_1000")
                .short("H")
                .long("si")
                .help("print sizes in powers of 1000"),
        )
        .arg(
            Arg::with_name("inodes")
                .short("i")
                .long("inodes")
                .help("list inode information instead of block usage")
        )
        .arg(
            // ignore this for now
            Arg::with_name("size_in_k")
                .short("k")
                .help("like --block-size=1K")
        )
        .arg(
            Arg::with_name("local")
                .short("l")
                .long("local")
                .help("limit listing to local file systems")
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .help("use the output format defined by FIELD_LIST, or print all fields if FIELD_LIST is omitted")
                .value_name("FIELD_LIST")
                .empty_values(true)
                .takes_value(true)
        )
        .arg(
            Arg::with_name("output_all_fields")
                .long("output-all-fields")
                .help("output all fields")
        )
        .arg(
            Arg::with_name("total")
                .long("total")
                .help("elide all entries insignificant to available space, and produce a grand total")
        )
        .arg(
            Arg::with_name("fs_type")
                .long("type")
                .short("t")
                .value_name("TYPE")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)
        )
        .arg(
            Arg::with_name("print_type")
                .long("print-type")
                .short("T")
                .help("print file system type")
        )
        .arg(
            Arg::with_name("excluded_fs_type")
                .long("exclude-type")
                .short("x")
                .value_name("EXCLUDE_TYPE")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)
        )
        .get_matches();
    options.show_all_fs = matches.is_present("all");
    options.human_readable =
        matches.is_present("human_readable_1024") || matches.is_present("human_readable_1000");
    options.human_readable_1024 = matches.is_present("human_readable_1024");
    options.inodes = matches.is_present("inodes");
    options.show_local_fs = matches.is_present("local");

    // parse output field list when necessary
    if matches.is_present("output") {
        options.field_list = matches
            .value_of("output")
            .unwrap()
            .split(",")
            .map(|x| x.to_owned())
            .collect();
        for field in options.field_list.iter() {
            println!("{}", field);
        }
    }

    options.print_grand_total = matches.is_present("total");
    options.show_fs_type = matches.is_present("print_type");
    options.output_all_fields = matches.is_present("output_all_fields");

    if matches.is_present("fs_type") {
        options.listed_fs = matches
            .values_of("fs_type")
            .unwrap()
            .map(|x| x.to_owned())
            .collect();
    }

    if matches.is_present("excluded_fs_type") {
        options.excluded_fs = matches
            .values_of("excluded_fs_type")
            .unwrap()
            .map(|x| x.to_owned())
            .collect();
    }

    options
}
