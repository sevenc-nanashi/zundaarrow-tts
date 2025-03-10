#[macro_export]
macro_rules! info {
    ($($args:tt)+) => {{
        use colored::Colorize;
        println!("{} {}", "i)".bold().cyan(), &format!($($args)*));
    }};
}

#[macro_export]
macro_rules! warn {
    ($($args:tt)+) => {{
        use colored::Colorize;
        println!("{} {}", "!)".bold().yellow(), &format!($($args)*));
    }};
}

#[macro_export]
macro_rules! error {
    ($($args:tt)+) => {{
        use colored::Colorize;
        println!("{} {}", "X)".bold().red(), &format!($($args)*));
    }};
}
