// use colored::Color;
use fern::colors::{Color, ColoredLevelConfig};

pub mod ast;
pub mod coms_proto;

pub fn logger_init() {
    let colors = ColoredLevelConfig::new()
        .trace(Color::Yellow)
        .debug(Color::Blue)
        .info(Color::Green)
        .warn(Color::Magenta)
        .error(Color::Red);

    #[cfg(debug_assertions)]
    let _res = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .filter(|metadata| metadata.target().starts_with("ducky_exec"))
        .chain(std::io::stderr())
        // .chain(fern::log_file("output.log")?)
        .apply();

    #[cfg(test)]
    let _res = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .filter(|metadata| metadata.target().starts_with("ducky_exec"))
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log")?)
        .apply();

    #[cfg(not(debug_assertions))]
    let _res = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                colors.color(record.level()),
                message
            ))
        })
        .filter(|metadata| metadata.target().starts_with("ducky_exec"))
        .chain(std::io::stderr())
        // .chain(fern::log_file("output.log")?)
        .apply();
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::DuckyScript;

    fn parse(source: &str) {
        let script = DuckyScript::from_source(source);

        assert!(script.is_ok());
    }

    #[test]
    fn generic_parser() {
        parse("LED_R\nDELAY 2000\nGUI r\nSTRINGLN notepad\nDELAY 5000\nLED_G\nSTRING H\nSHIFT ello, World!\nENTER\nREM foobar\nLED_OFF\nSHIFT UP");
        parse("LED_R\nDELAY 2000\nGUI r\nSTRINGLN notepad\nDELAY 5000\nLED_G\nSTRING H\nSHIFT ello, World!\nENTER\nREM foobar\nLED_OFF\nSHIFT UP\n");
        parse("\nLED_R\nDELAY 2000\nGUI r\nSTRINGLN notepad\nDELAY 5000\nLED_G\nSTRING H\nSHIFT ello, World!\nENTER\nREM foobar\nLED_OFF\nSHIFT UP");
        parse("\nLED_R\nDELAY 2000\nGUI r\nSTRINGLN notepad\nDELAY 5000\nLED_G\nSTRING H\nSHIFT ello, World!\nENTER\nREM foobar\nLED_OFF\nSHIFT UP\n");

        // assert!(1 == 2);
    }

    #[test]
    fn blocks_and_multiline() {
        parse("\nSTRINGLN\n\tline1\nEND_STRINGLN");
        parse("\nSTRINGLN\nline1\nEND_STRINGLN\n");
        parse("\nREM_BLOCK\nline1\nline2\nEND_REM\n");
        parse("\nREM_BLOCK\nline1\nline2\nEND_REM\nREM foobar");
        parse("\nGUI\n");

        // assert!(1 == 2);
    }
}
