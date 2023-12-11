mod logger;

use crate::logger::Logger;
fn main() {
    let mut logger = Logger::new();
    logger.set_log_level(logger::LogLevel::Info);
    logger.log("this is an info message");
    logger.warning("this is a warning message");
    logger.error("this is a error message");

    let entires = logger.get_log_entires();
    println!("this is log messages...");
    entires.iter().for_each(|x| {
        println!("{}", x.message);
    });
}
