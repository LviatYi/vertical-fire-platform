use crate::constant::log::*;
use crate::extract::extract_operation_info::{ExtractOperationInfo, OperationStatus};
use crossterm::cursor::{MoveUp, RestorePosition, SavePosition};
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use formatx::formatx;
use std::io;
use std::io::{Stdout, Write};

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
                let _ = execute!(
                    stdout,
                    SetForegroundColor(Color::Yellow),
                    Print("   "),
                    Print(OPERATION_CLEAN),
                    ResetColor,
                );
            }
            OperationStatus::Done(Some(d)) => {
                let _ = execute!(
                    stdout,
                    SetForegroundColor(if working { Color::Green } else { Color::Grey }),
                    Print(" "),
                    Print(formatx!(RESULT_CLEAN, d).unwrap_or_default()),
                    ResetColor,
                );
            }
            _ => {}
        }

        if let OperationStatus::Done(_) = status.clean_state {
            match status.extract_state {
                OperationStatus::Pending => {
                    let _ = execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        Print("   "),
                        Print(OPERATION_EXTRACT),
                        ResetColor,
                    );
                }
                OperationStatus::Done(Some(d)) => {
                    let _ = execute!(
                        stdout,
                        SetForegroundColor(if working { Color::Green } else { Color::Grey }),
                        Print(" "),
                        Print(formatx!(RESULT_EXTRACT, d).unwrap_or_default()),
                        ResetColor,
                    );
                }
                _ => {}
            }
        }

        if let OperationStatus::Done(_) = status.extract_state {
            match status.mend_state {
                OperationStatus::Pending => {
                    let _ = execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        Print("   "),
                        Print(OPERATION_MEND),
                        ResetColor,
                    );
                }
                OperationStatus::Done(Some(d)) => {
                    let _ = execute!(
                        stdout,
                        SetForegroundColor(if working { Color::Green } else { Color::Grey }),
                        Print(" "),
                        Print(formatx!(RESULT_MEND, d).unwrap_or_default()),
                        ResetColor,
                    );
                }
                _ => {}
            }
        }

        let _ = execute!(stdout, RestorePosition);

        Ok(())
    }
}
