use crate::coms_proto::LedState;
use anyhow::bail;
use log::*;
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use std::{thread::sleep, time::Duration};

pub type Result = anyhow::Result<()>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct DuckyScript;

impl DuckyScript {
    pub fn from_source(source: &str) -> Result {
        trace!("Compiling the source: {:?}", source);
        let script = Self::parse(Rule::SCRIPT, source); // parser::parse(source).unwrap();
                                                        // trace!("script => {:?}", script);
        #[cfg(test)]
        {
            print!("Compiling the source: {:?}", source);
            print!("script => {:#?}", script);
            // trace!("flattened script => {:#?}", script?.flatten());
            Ok(())
        }
        #[cfg(not(test))]
        {
            trace!("script => {:?}", script);
            Self::exec(script?)
        }
    }

    fn exec(script: Pairs<Rule>) -> anyhow::Result<()> {
        // NOTE: I could make this handle everything with no healper functions by making it recurse
        // on line.into_inner(), how ever this would be significantly more unreadable.
        for line in script {
            match line.as_rule() {
                // Rule::EOI => break,
                // Rule::WHITESPACE => {}
                Rule::NEWLINE => {}
                Rule::DELAY => delay(line)?,
                // Rule::LED => led(line)?,
                Rule::LED_R => led(LedState::RED)?,
                Rule::LED_G => led(LedState::GREEN)?,
                Rule::LED_OFF => led(LedState::OFF)?,
                Rule::LOCK_KEYS => toggle_lock_key(line)?,
                Rule::STRINGLN_BLOCK => stringln_block(line)?,
                // Rule::
                _ => {}
            }
        }

        Ok(())
    }
}

fn stringln_block(line: Pair<Rule>) -> Result {
    // TODO: Write

    Ok(())
}

fn toggle_lock_key(line: Pair<Rule>) -> Result {
    // TODO: Write

    Ok(())
}

// fn led(line: Pair<Rule>) -> Result {
//     if let Some(led_state) = line.into_inner().next() {
//         match led_state.as_rule() {
//             Rule::LED_R => {}
//             Rule::LED_G => {}
//             Rule::LED_OFF => {}
//             _ => unreachable!(
//                 "Rule::LED is a combination rule and can only contain Rule::LED_( R | G | OFF )"
//             ),
//         }
//     } else {
//         bail!("delay takes a argument");
//     }
//
//     Ok(())
// }

fn led(state: LedState) -> Result {
    println!("setting led to be {state:?}");

    Ok(())
}

fn delay(line: Pair<Rule>) -> Result {
    if let Some(delay_amt) = line.into_inner().next() {
        if let Ok(time) = delay_amt.as_str().parse() {
            sleep(Duration::from_millis(time));
            Ok(())
        } else {
            bail!("delay must be a positive number and nothing else.");
        }
    } else {
        bail!("delay takes a argument");
    }
}
