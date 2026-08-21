use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use crate::app::App;
use crate::cli::CliRoute;
use crate::domain::GameCatalog;
use crate::history::HistoryStore;
use crate::ui::Renderer;

const INPUT_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Runs the interactive terminal and guarantees best-effort terminal cleanup.
pub fn run_tui(
    catalog: GameCatalog,
    route: CliRoute,
    history: Box<dyn HistoryStore>,
) -> io::Result<()> {
    let mut terminal = TerminalSession::enter()?;
    let mut renderer = Renderer::new();
    let run_result = run_event_loop(
        terminal.output(),
        &mut renderer,
        App::new(catalog, route, history),
    );
    let restore_result = terminal.restore();
    prefer_runtime_error(run_result, restore_result)
}

fn run_event_loop(output: &mut Stdout, renderer: &mut Renderer, mut app: App) -> io::Result<()> {
    while !app.should_quit() {
        renderer.draw(output, &app)?;
        if event::poll(INPUT_POLL_INTERVAL)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }
        app.on_tick();
    }
    Ok(())
}

/// Owns terminal mode for the complete interactive lifetime.
///
/// `Drop` is deliberately best-effort so a panic or early `?` does not leave
/// the user's shell in raw mode or on the alternate screen.
struct TerminalSession {
    output: Stdout,
    active: bool,
}

impl TerminalSession {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut output = io::stdout();
        if let Err(error) = execute!(output, EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            let _ = execute!(output, ResetColor, Show, LeaveAlternateScreen);
            return Err(error);
        }
        Ok(Self {
            output,
            active: true,
        })
    }

    fn output(&mut self) -> &mut Stdout {
        &mut self.output
    }

    fn restore(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        let raw_mode_result = disable_raw_mode();
        let screen_result = execute!(self.output, ResetColor, Show, LeaveAlternateScreen);
        self.active = false;
        raw_mode_result.and(screen_result)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn prefer_runtime_error(
    runtime_result: io::Result<()>,
    restore_result: io::Result<()>,
) -> io::Result<()> {
    match runtime_result {
        Err(error) => Err(error),
        Ok(()) => restore_result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_error_is_not_hidden_by_cleanup_error() {
        let runtime_error = io::Error::other("event loop failed");
        let cleanup_error = io::Error::other("cleanup failed");
        let error = prefer_runtime_error(Err(runtime_error), Err(cleanup_error))
            .expect_err("runtime error");
        assert_eq!(error.to_string(), "event loop failed");
    }

    #[test]
    fn cleanup_error_is_returned_after_successful_runtime() {
        let cleanup_error = io::Error::other("cleanup failed");
        let error = prefer_runtime_error(Ok(()), Err(cleanup_error)).expect_err("cleanup error");
        assert_eq!(error.to_string(), "cleanup failed");
    }
}
