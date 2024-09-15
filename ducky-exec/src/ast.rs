use crate::coms_proto::{Command, LedState, SpecialKey};
use anyhow::bail;
use log::*;
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use serialport::SerialPortBuilder;
use std::{io::Write, thread::sleep, time::Duration};

pub type Result = anyhow::Result<()>;

#[derive(Debug, Clone, PartialEq, Eq, pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct DuckyScript {
    port: SerialPortBuilder,
    pub line: usize,
}

impl DuckyScript {
    pub fn new(port_adr: &str) -> Self {
        Self {
            port: serialport::new(port_adr, 115200),
            line: 0,
        }
    }

    pub fn get_len(source: &str) -> anyhow::Result<usize> {
        let script = Self::parse(Rule::SCRIPT, source);

        #[cfg(test)]
        {
            print!("Compiling the source: {:?}", source);
            print!("script => {:#?}", script);
            // trace!("flattened script => {:#?}", script?.flatten());
        }

        Ok(script?.len() - 1)
    }

    pub fn from_source(&mut self, source: &str) -> Result {
        trace!("Compiling the source: {:?}", source);
        let script = Self::parse(Rule::SCRIPT, source);
        trace!("script => {:?}", script);
        self.exec(script?)
    }

    fn exec(&mut self, script: Pairs<Rule>) -> Result {
        // NOTE: I could make this handle everything with no healper functions by making it recurse
        // on line.into_inner(), how ever this would be significantly more unreadable.
        for line in script {
            if line.as_rule() == Rule::EOI {
                break;
            } else {
                self.line = line.line_col().0
            }

            self.handle_rule(line)?;
        }

        Ok(())
    }

    fn handle_rule(&mut self, rule: Pair<Rule>) -> Result {
        match rule.as_rule() {
            Rule::EOI => {}
            Rule::WHITESPACE => {}
            Rule::NEWLINE => {}
            Rule::DELAY => self.delay(rule)?,
            // Rule::LED => led(line)?,
            Rule::LED_R => self.led(LedState::RED)?,
            Rule::LED_G => self.led(LedState::GREEN)?,
            Rule::LED_OFF => self.led(LedState::OFF)?,
            Rule::LOCK_KEYS => {} // Self::exec(rule.into_inner())?,
            // Rule::STRING => trigger_keys(line)?,
            Rule::STRING => self.exec(rule.into_inner())?,
            Rule::STRINGLN => {
                self.exec(rule.into_inner())?;
                self.default_delay_key_stroke();
                self.hit_key(SpecialKey::Enter);
            }
            Rule::STRING_BLOCK => {
                for line in rule.into_inner() {
                    self.handle_rule(line)?;
                    self.default_delay_key_stroke();
                }
            }

            Rule::STRINGLN_BLOCK => {
                for line in rule.into_inner() {
                    self.handle_rule(line)?;
                    self.default_delay_key_stroke();
                    self.hit_key(SpecialKey::Enter);
                }
                // stringln_block(line)?,
            }
            Rule::text => self.type_text(rule.as_str()),
            Rule::REM
            | Rule::single_line_rem
            | Rule::multi_line_rem
            | Rule::rem_block_start
            | Rule::rem_block_end => {}
            Rule::cursor_keys | Rule::sys_mods | Rule::modifier => {}
            Rule::UP => self.hit_key(SpecialKey::Up),
            Rule::DOWN => self.hit_key(SpecialKey::Down),
            Rule::LEFT => self.hit_key(SpecialKey::Left),
            Rule::RIGHT => self.hit_key(SpecialKey::Right),
            Rule::PAGEUP => self.hit_key(SpecialKey::PgUp),
            Rule::PAGEDOWN => self.hit_key(SpecialKey::PgDown),
            Rule::HOME => self.hit_key(SpecialKey::Home),
            Rule::END => self.hit_key(SpecialKey::End),
            Rule::INSERT => self.hit_key(SpecialKey::Ins),
            Rule::DELETE => self.hit_key(SpecialKey::Del),
            Rule::BACKSPACE => self.hit_key(SpecialKey::BackSpace),
            Rule::TAB => self.hit_key(SpecialKey::Tab),
            Rule::SPACE => self.hit_key(SpecialKey::Space),
            Rule::ENTER => self.hit_key(SpecialKey::Enter),
            Rule::ESCAPE => self.hit_key(SpecialKey::Esc),
            Rule::PAUSE_BREAK => self.hit_key(SpecialKey::PauseBreak),
            Rule::PRINTSCREEN => self.hit_key(SpecialKey::PrntScrn),
            Rule::MENU_APP => self.hit_key(SpecialKey::Menu),
            Rule::F1 => self.hit_key(SpecialKey::F1),
            Rule::F2 => self.hit_key(SpecialKey::F2),
            Rule::F3 => self.hit_key(SpecialKey::F3),
            Rule::F4 => self.hit_key(SpecialKey::F4),
            Rule::F5 => self.hit_key(SpecialKey::F5),
            Rule::F6 => self.hit_key(SpecialKey::F6),
            Rule::F7 => self.hit_key(SpecialKey::F7),
            Rule::F8 => self.hit_key(SpecialKey::F8),
            Rule::F9 => self.hit_key(SpecialKey::F9),
            Rule::F10 => self.hit_key(SpecialKey::F10),
            Rule::F11 => self.hit_key(SpecialKey::F11),
            Rule::F12 => self.hit_key(SpecialKey::F12),
            Rule::SHIFT => self.hit_key(SpecialKey::LeftShift),
            Rule::ALT => self.hit_key(SpecialKey::LeftAlt),
            Rule::CONTROL => self.hit_key(SpecialKey::LeftCtrl),
            Rule::COMMAND | Rule::WINDOWS => self.hit_key(SpecialKey::LeftSuper),
            Rule::key_mod_compbo => self.key_mod_combo(rule.into_inner())?,
            // Rule:: => hit_key(SpecialKey::),
            // Rule:: => hit_key(SpecialKey::),
            // Rule:: => hit_key(SpecialKey::),
            // Rule:: => hit_key(SpecialKey::),
            Rule::INJECT_MOD => self.exec(rule.into_inner())?,
            Rule::CAPS_LOCK => self.send_command(Command::TriggerKey(SpecialKey::CapsLock)),
            Rule::NUM_LOCK => self.send_command(Command::TriggerKey(SpecialKey::NumLock)),
            Rule::SCROLLLOCK => self.send_command(Command::TriggerKey(SpecialKey::ScrollLock)),
            Rule::DELAY_KW => {}
            Rule::DELAY_ARG => {}
            Rule::INJECT_KW => {}
            Rule::lines | Rule::SCRIPT => {}
        }

        Ok(())
    }

    fn key_mod_combo(&self, rules: Pairs<Rule>) -> Result {
        let mut to_release: Vec<SpecialKey> = Vec::with_capacity(rules.len());

        for rule in rules {
            let key = match rule.as_rule() {
                Rule::UP => self.press_key(SpecialKey::Up),
                Rule::DOWN => self.press_key(SpecialKey::Down),
                Rule::LEFT => self.press_key(SpecialKey::Left),
                Rule::RIGHT => self.press_key(SpecialKey::Right),
                Rule::PAGEUP => self.press_key(SpecialKey::PgUp),
                Rule::PAGEDOWN => self.press_key(SpecialKey::PgDown),
                Rule::HOME => self.press_key(SpecialKey::Home),
                Rule::END => self.press_key(SpecialKey::End),
                Rule::INSERT => self.press_key(SpecialKey::Ins),
                Rule::DELETE => self.press_key(SpecialKey::Del),
                Rule::BACKSPACE => self.press_key(SpecialKey::BackSpace),
                Rule::TAB => self.press_key(SpecialKey::Tab),
                Rule::SPACE => self.press_key(SpecialKey::Space),
                Rule::ENTER => self.press_key(SpecialKey::Enter),
                Rule::ESCAPE => self.press_key(SpecialKey::Esc),
                Rule::PAUSE_BREAK => self.press_key(SpecialKey::PauseBreak),
                Rule::PRINTSCREEN => self.press_key(SpecialKey::PrntScrn),
                Rule::MENU_APP => self.press_key(SpecialKey::Menu),
                Rule::F1 => self.press_key(SpecialKey::F1),
                Rule::F2 => self.press_key(SpecialKey::F2),
                Rule::F3 => self.press_key(SpecialKey::F3),
                Rule::F4 => self.press_key(SpecialKey::F4),
                Rule::F5 => self.press_key(SpecialKey::F5),
                Rule::F6 => self.press_key(SpecialKey::F6),
                Rule::F7 => self.press_key(SpecialKey::F7),
                Rule::F8 => self.press_key(SpecialKey::F8),
                Rule::F9 => self.press_key(SpecialKey::F9),
                Rule::F10 => self.press_key(SpecialKey::F10),
                Rule::F11 => self.press_key(SpecialKey::F11),
                Rule::F12 => self.press_key(SpecialKey::F12),
                Rule::SHIFT => self.press_key(SpecialKey::LeftShift),
                Rule::ALT => self.press_key(SpecialKey::LeftAlt),
                Rule::CONTROL => self.press_key(SpecialKey::LeftCtrl),
                Rule::COMMAND | Rule::WINDOWS => self.press_key(SpecialKey::LeftSuper),
                Rule::text => {
                    self.type_text(rule.as_str());

                    None
                }
                _ => None,
            };

            if let Some(key) = key {
                to_release.push(key);
            }

            self.default_delay_key_stroke()
        }

        to_release
            .into_iter()
            .for_each(|key| self.send_command(Command::ReleaseKey(key)));

        Ok(())
    }

    fn send_command(&self, cmd: Command) {
        if let Ok(message) = serde_cbor::to_vec(&cmd) {
            if let Ok(mut port) = self.port.clone().open_native() {
                if let Err(e) = port.write_all(&message) {
                    error!("sending data over uart failed with error: {e}");
                }
            } else {
                error!("failed to open serial port");
            }
        } else {
            error!("failed to serialize command");
        }
    }

    fn hit_key(&self, key: SpecialKey) {
        self.send_command(Command::TriggerKey(key));
    }

    fn press_key(&self, key: SpecialKey) -> Option<SpecialKey> {
        self.send_command(Command::HoldKey(key));

        Some(key)
    }

    fn type_text(&self, text: &str) {
        text.as_bytes().into_iter().for_each(|char| {
            self.send_command(Command::TypeChar(*char));
            self.default_delay_up_down()
        })
    }

    fn led(&self, state: LedState) -> Result {
        println!("setting led to be {state:?}");

        Ok(())
    }

    fn delay(&self, line: Pair<Rule>) -> Result {
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

    fn default_delay_up_down(&self) {
        sleep(Duration::from_millis(25));
    }

    fn default_delay_key_stroke(&self) {
        sleep(Duration::from_millis(50));
    }
}
