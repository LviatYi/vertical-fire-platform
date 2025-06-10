use crate::constant::log::*;
use crate::extract::extract_operation_info::{ExtractOperationInfo, OperationStatus};
use crossterm::cursor::{MoveUp, RestorePosition, SavePosition};
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use formatx::formatx;
use std::io;
use std::io::{Stdout, Write};
use win_toast_notify::WinToastNotify;

pub struct VfpPrettyLogger;

impl VfpPrettyLogger {
    pub fn apply_for(stdout: &mut Stdout, line_count: u32) -> Self {
        for _i in 0..line_count {
            println!();
        }

        let _ = stdout.flush();

        Self
    }

    pub fn pretty_log_operation_status(
        &self,
        stdout: &mut Stdout,
        index: u32,
        all_count: u32,
        status: &ExtractOperationInfo,
    ) -> io::Result<()> {
        let _ = execute!(
            stdout,
            SavePosition,
            MoveUp((all_count - index) as u16),
            Clear(ClearType::CurrentLine),
        );

        let working = !status.is_done();
        let error = status.has_error();

        if error {
            let _ = execute!(
                stdout,
                SetForegroundColor(Color::Red),
                Print(formatx!(OPERATION_FAILED, index).unwrap_or_default()),
                Print("   "),
                SetForegroundColor(Color::DarkRed),
                Print(status.first_error_message()),
                ResetColor,
            );
        } else if working {
            let _ = execute!(
                stdout,
                SetForegroundColor(Color::Yellow),
                Print(formatx!(OPERATION_TITLE, index).unwrap_or_default()),
                ResetColor,
            );
        } else {
            let _ = execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print(formatx!(OPERATION_FINISHED, index).unwrap_or_default()),
                Print("   "),
                SetForegroundColor(Color::DarkGreen),
                Print(formatx!(OPERATION_ALL_COST, status.all_cost()).unwrap_or_default()),
                ResetColor,
            );
        }

        match status.clean_state {
            OperationStatus::Pending => {
                colored_print(
                    stdout,
                    ThemeColor::Warn,
                    format!("   {}", OPERATION_CLEAN).as_str(),
                );
            }
            OperationStatus::Done(Some(d)) => {
                colored_print(
                    stdout,
                    if working {
                        ThemeColor::Success
                    } else {
                        ThemeColor::Second
                    },
                    format!(" {}", formatx!(RESULT_CLEAN, d).unwrap_or_default()).as_str(),
                );
            }
            _ => {}
        }

        if let OperationStatus::Done(_) = status.clean_state {
            match status.extract_state {
                OperationStatus::Pending => {
                    colored_print(
                        stdout,
                        ThemeColor::Warn,
                        format!("   {}", OPERATION_EXTRACT).as_str(),
                    );
                }
                OperationStatus::Done(Some(d)) => {
                    colored_print(
                        stdout,
                        if working {
                            ThemeColor::Success
                        } else {
                            ThemeColor::Second
                        },
                        format!(" {}", formatx!(RESULT_EXTRACT, d).unwrap_or_default()).as_str(),
                    );
                }
                _ => {}
            }
        }

        if let OperationStatus::Done(_) = status.extract_state {
            match status.mend_state {
                OperationStatus::Pending => {
                    colored_print(
                        stdout,
                        ThemeColor::Warn,
                        format!("   {}", OPERATION_MEND).as_str(),
                    );
                }
                OperationStatus::Done(Some(d)) => {
                    colored_print(
                        stdout,
                        if working {
                            ThemeColor::Success
                        } else {
                            ThemeColor::Second
                        },
                        format!(" {}", formatx!(RESULT_MEND, d).unwrap_or_default()).as_str(),
                    );
                }
                _ => {}
            }
        }

        let _ = execute!(stdout, RestorePosition);

        Ok(())
    }
}

pub enum ThemeColor {
    Main,
    Second,
    Success,
    Warn,
    Error,
}

impl ThemeColor {
    pub fn to_color(&self) -> Color {
        match self {
            ThemeColor::Main => Color::White,
            ThemeColor::Second => Color::DarkGrey,
            ThemeColor::Success => Color::Green,
            ThemeColor::Warn => Color::Yellow,
            ThemeColor::Error => Color::Red,
        }
    }
}

pub fn colored_print(stdout: &mut Stdout, color: ThemeColor, content: &str) {
    let _ = execute!(
        stdout,
        SetForegroundColor(color.to_color()),
        Print(content),
        ResetColor,
    );
}

pub fn colored_println(stdout: &mut Stdout, color: ThemeColor, content: &str) {
    let _ = execute!(
        stdout,
        SetForegroundColor(color.to_color()),
        Print(format!("{}\n", content)),
        ResetColor,
    );
}

pub fn clean_one_line(stdout: &mut Stdout) {
    let _ = execute!(stdout, MoveUp(1), Clear(ClearType::CurrentLine),);
}

pub fn toast(title: &str, msg: Vec<&str>) {
    WinToastNotify::new()
        .set_title(&format!("V-F Platform | {}", title))
        .set_messages(msg)
        .show()
        .expect(ERR_TOAST_SHOW_FAILED)
}
