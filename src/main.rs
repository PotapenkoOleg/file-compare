use crate::clap_parser::Args;
use crate::trie::ternary_trie::TernarySearchTrie;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

mod clap_parser;
mod trie;

fn main() {
    let args = Args::parse();

    // TODO: handle duplicate keys
    let mut symbol_table_1 = TernarySearchTrie::<u32>::new();
    build_symbol_table(&args.first, &mut symbol_table_1, args.ignore_case);

    let mut symbol_table_2 = TernarySearchTrie::<u32>::new();
    build_symbol_table(&args.second, &mut symbol_table_2, args.ignore_case);

    let mut words_in_first_not_in_second: Vec<(u32, String)> = Vec::new();
    for key in &symbol_table_1.get_all_keys() {
        if !symbol_table_2.contains(key.as_ref()) {
            let value = symbol_table_1.get(key.as_ref()).unwrap();
            words_in_first_not_in_second.push((value, key.clone()))
        }
    }

    let mut words_in_second_not_in_first: Vec<(u32, String)> = Vec::new();
    for key in &symbol_table_2.get_all_keys() {
        if !symbol_table_1.contains(key.as_ref()) {
            let value = symbol_table_2.get(key.as_ref()).unwrap();
            words_in_second_not_in_first.push((value, key.clone()))
        }
    }

    words_in_first_not_in_second.sort_by_key(|k| k.0);
    words_in_second_not_in_first.sort_by_key(|k| k.0);

    if args.render_html {
        render_html_output(
            &args.first,
            &args.second,
            &words_in_first_not_in_second,
            &words_in_second_not_in_first,
        );
    } else {
        render_text_output(
            &args.first,
            &args.second,
            &words_in_first_not_in_second,
            &words_in_second_not_in_first,
        );
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn build_symbol_table(
    filename: &str,
    symbol_table: &mut TernarySearchTrie<u32>,
    ignore_case: bool,
) {
    if let Ok(lines) = read_lines(filename) {
        for (index, line) in lines.enumerate() {
            if let Ok(current_line) = line {
                if ignore_case {
                    symbol_table.put(current_line.to_uppercase(), index as u32);
                } else {
                    symbol_table.put(current_line, index as u32);
                }
            }
        }
    }
}

fn build_separator() -> String {
    let template = "*";
    let n = 80;
    template.repeat(n)
}

fn print_separator() {
    println!("{}", build_separator());
}

fn render_text_output(
    first: &str,
    second: &str,
    words_in_first_not_in_second: &Vec<(u32, String)>,
    words_in_second_not_in_first: &Vec<(u32, String)>,
) {
    print_separator();
    println!(
        "LINES IN FIRST ({}) FILE, BUT NOT IN SECOND ({})",
        first, second
    );
    print_separator();
    for (num, text) in words_in_first_not_in_second {
        println!("line {}: {}", num, text);
    }
    print_separator();
    println!("TOTAL: {}", words_in_first_not_in_second.len());
    print_separator();
    println!(
        "LINES IN SECOND ({}) FILE, BUT NOT IN FIRST ({})",
        second, first
    );
    print_separator();
    for (num, text) in words_in_second_not_in_first {
        println!("line {}: {}", num, text);
    }
    print_separator();
    println!("TOTAL: {}", words_in_second_not_in_first.len());
    print_separator();
}

fn render_html_output(
    first: &str,
    second: &str,
    words_in_first_not_in_second: &Vec<(u32, String)>,
    words_in_second_not_in_first: &Vec<(u32, String)>,
) {
    println!("<html>");
    println!("<head>");
    println!("<style>");
    println!(".table-section {{ background-color: #A6AEBF;  }} ");
    println!(".table-header {{ background-color: #C5D3E8; }} ");
    println!(".table-body {{ background-color: #D0E8C5; }} ");
    println!(".table-footer {{ background-color: #FFF8DE; }} ");
    println!("</style>");
    println!("</head>");
    println!("<body>");
    println!("<table border=\"1\">");
    println!(
        "<tr class=table-section><td colspan=2>LINES IN FIRST (<b>{}</b>) FILE, BUT NOT IN SECOND (<b>{}</b>)</td></tr>",
        first, second
    );
    println!("<tr class=table-header><th>Line Number</th><th>Text</th></tr>");
    for (num, text) in words_in_first_not_in_second {
        println!("<tr class=table-body><td>{}</td><td>{}</td></tr>", num, text);
    }
    println!(
        "<tr class=table-footer><td colspan=2>TOTAL: {}</td></tr>",
        words_in_first_not_in_second.len()
    );
    println!(
        "<tr class=table-section><td colspan=2>LINES IN SECOND (<b>{}</b>) FILE, BUT NOT IN FIRST (<b>{}</b>)</td></tr>",
        second, first
    );
    println!("<tr class=table-header><th>Line Number</th><th>Text</th></tr>");
    for (num, text) in words_in_second_not_in_first {
        println!("<tr class=table-body><td>{}</td><td>{}</td></tr>", num, text);
    }
    println!(
        "<tr class=table-footer><td colspan=2>TOTAL: {}</td></tr>",
        words_in_second_not_in_first.len()
    );
    println!("</table>");
    println!("</body></html>");
}
