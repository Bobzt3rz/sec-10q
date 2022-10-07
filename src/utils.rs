use console::style;
use indicatif::{ProgressBar, ProgressStyle};

pub fn without_dash(string: &String) -> String {
    return string.replace('-', "");
}

pub fn get_bar_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
    )
    .unwrap()
}

pub fn get_bar(size: usize) -> ProgressBar {
    let pb = ProgressBar::new(size.try_into().unwrap());
    pb.set_style(get_bar_style());
    pb
}

pub fn print_loading(text: &str, current: i32, total: i32) -> i32 {
    println!(
        "{} {}...",
        style(format!("[{}/{}]", current, total)).bold().dim(),
        text,
    );
    current + 1
}
