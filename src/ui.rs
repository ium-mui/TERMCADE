use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{self, Clear, ClearType};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::{App, Screen};
use crate::blackjack::{Card, Suit};
use crate::breakout::{
    BOARD_HEIGHT as BREAKOUT_BOARD_HEIGHT, BOARD_WIDTH as BREAKOUT_BOARD_WIDTH, PADDLE_WIDTH,
};
use crate::connect_four::{
    CONNECT_FOUR_HEIGHT, CONNECT_FOUR_WIDTH, ConnectFourCell, ConnectFourOutcome,
};
use crate::domain::{GameDefinition, GameKind, StageDefinition};
use crate::game_2048::{GRID_SIZE, Game2048Session};
use crate::holdem::HoldemAiAction;
use crate::memory_match::MemoryCardState;
use crate::minesweeper::{CellState as MineCellState, MinesweeperOutcome};
use crate::roulette::{RouletteColor, WHEEL_NUMBERS};
use crate::round::{RoundSession, RoundStatus};
use crate::snake::{BOARD_HEIGHT, BOARD_WIDTH, Direction, Point};
use crate::sudoku::SUDOKU_SIZE;
use crate::tictactoe::Cell as TicTacToeCell;

const BG: Color = Color::Rgb { r: 7, g: 11, b: 18 };
const HEADER_BG: Color = Color::Rgb {
    r: 12,
    g: 19,
    b: 31,
};
const SURFACE: Color = Color::Rgb {
    r: 14,
    g: 22,
    b: 35,
};
const PANEL: Color = Color::Rgb {
    r: 18,
    g: 29,
    b: 45,
};
const PANEL_ALT: Color = Color::Rgb {
    r: 27,
    g: 42,
    b: 63,
};
const PANEL_HOVER: Color = Color::Rgb {
    r: 35,
    g: 61,
    b: 84,
};
const GRID: Color = Color::Rgb {
    r: 26,
    g: 42,
    b: 61,
};
const BORDER: Color = Color::Rgb {
    r: 47,
    g: 72,
    b: 96,
};
const BORDER_BRIGHT: Color = Color::Rgb {
    r: 79,
    g: 123,
    b: 151,
};
const TEXT: Color = Color::Rgb {
    r: 235,
    g: 242,
    b: 250,
};
const MUTED: Color = Color::Rgb {
    r: 132,
    g: 158,
    b: 184,
};
const CYAN: Color = Color::Rgb {
    r: 92,
    g: 224,
    b: 214,
};
const BLUE: Color = Color::Rgb {
    r: 96,
    g: 175,
    b: 255,
};
const GREEN: Color = Color::Rgb {
    r: 102,
    g: 224,
    b: 150,
};
const YELLOW: Color = Color::Rgb {
    r: 255,
    g: 200,
    b: 86,
};
const RED: Color = Color::Rgb {
    r: 255,
    g: 103,
    b: 123,
};
const PURPLE: Color = Color::Rgb {
    r: 178,
    g: 137,
    b: 255,
};
const PINK: Color = Color::Rgb {
    r: 255,
    g: 125,
    b: 180,
};

pub const MIN_TERMINAL_WIDTH: u16 = 64;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Paint {
    foreground: Color,
    background: Color,
    bold: bool,
}

impl Paint {
    const fn new(foreground: Color, background: Color, bold: bool) -> Self {
        Self {
            foreground,
            background,
            bold,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Cell {
    character: char,
    paint: Paint,
}

impl Cell {
    const fn blank(paint: Paint) -> Self {
        Self {
            character: ' ',
            paint,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Rect {
    const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn inset(self, amount: u16) -> Self {
        Self {
            x: self.x.saturating_add(amount),
            y: self.y.saturating_add(amount),
            width: self.width.saturating_sub(amount.saturating_mul(2)),
            height: self.height.saturating_sub(amount.saturating_mul(2)),
        }
    }
}

struct Buffer {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl Buffer {
    fn new(width: u16, height: u16) -> Self {
        let paint = Paint::new(TEXT, BG, false);
        Self {
            width,
            height,
            cells: vec![Cell::blank(paint); usize::from(width) * usize::from(height)],
        }
    }

    fn index(&self, x: u16, y: u16) -> usize {
        usize::from(y) * usize::from(self.width) + usize::from(x)
    }

    fn set(&mut self, x: u16, y: u16, character: char, paint: Paint) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = self.index(x, y);
        self.cells[index] = Cell { character, paint };
    }

    fn set_continuation(&mut self, x: u16, y: u16, paint: Paint) {
        self.set(x, y, '\0', paint);
    }

    fn fill(&mut self, rect: Rect, paint: Paint) {
        for y in rect.y..rect.y.saturating_add(rect.height).min(self.height) {
            for x in rect.x..rect.x.saturating_add(rect.width).min(self.width) {
                self.set(x, y, ' ', paint);
            }
        }
    }

    fn text(&mut self, x: u16, y: u16, value: &str, paint: Paint) -> u16 {
        let mut cursor = x;
        for character in value.chars() {
            let width = UnicodeWidthChar::width(character).unwrap_or(1) as u16;
            if width == 0 {
                continue;
            }
            if cursor >= self.width || y >= self.height {
                break;
            }
            self.set(cursor, y, character, paint);
            if width == 2 {
                self.set_continuation(cursor.saturating_add(1), y, paint);
            }
            cursor = cursor.saturating_add(width);
        }
        cursor.saturating_sub(x)
    }

    fn centered_text(&mut self, rect: Rect, y: u16, value: &str, paint: Paint) {
        let width = value.width() as u16;
        let x = rect.x.saturating_add(rect.width.saturating_sub(width) / 2);
        self.text(x, y, value, paint);
    }

    fn line(&mut self, x: u16, y: u16, width: u16, character: char, paint: Paint) {
        for offset in 0..width {
            self.set(x.saturating_add(offset), y, character, paint);
        }
    }

    fn panel(&mut self, rect: Rect, title: &str, border: Color, background: Color) {
        if rect.width == 0 || rect.height == 0 {
            return;
        }
        let fill = Paint::new(TEXT, background, false);
        self.fill(rect, fill);
        if rect.width < 2 || rect.height < 2 {
            return;
        }
        let edge = Paint::new(border, background, false);
        self.set(rect.x, rect.y, '╭', edge);
        self.set(rect.x.saturating_add(rect.width - 1), rect.y, '╮', edge);
        self.set(rect.x, rect.y.saturating_add(rect.height - 1), '╰', edge);
        self.set(
            rect.x.saturating_add(rect.width - 1),
            rect.y.saturating_add(rect.height - 1),
            '╯',
            edge,
        );
        self.line(rect.x.saturating_add(1), rect.y, rect.width - 2, '─', edge);
        self.line(
            rect.x.saturating_add(1),
            rect.y.saturating_add(rect.height - 1),
            rect.width - 2,
            '─',
            edge,
        );
        for y in rect.y.saturating_add(1)..rect.y.saturating_add(rect.height - 1) {
            self.set(rect.x, y, '│', edge);
            self.set(rect.x.saturating_add(rect.width - 1), y, '│', edge);
        }
        if !title.is_empty() && rect.width > 8 {
            self.text(
                rect.x.saturating_add(2),
                rect.y,
                title,
                Paint::new(CYAN, background, true),
            );
        }
    }
}

pub struct Renderer {
    previous: Option<Buffer>,
}

impl Renderer {
    pub fn new() -> Self {
        crossterm::style::force_color_output(true);
        Self { previous: None }
    }

    pub fn draw(&mut self, output: &mut impl Write, app: &App) -> io::Result<()> {
        let (width, height) = terminal::size()?;
        self.draw_at(output, app, width, height)
    }

    /// Renders at a fixed size for automated tests and non-interactive hosts.
    pub fn draw_at(
        &mut self,
        output: &mut impl Write,
        app: &App,
        width: u16,
        height: u16,
    ) -> io::Result<()> {
        let mut buffer = Buffer::new(width, height);
        let area = Rect::new(0, 0, width, height);
        if width < MIN_TERMINAL_WIDTH || height < MIN_TERMINAL_HEIGHT {
            draw_small(&mut buffer, area);
        } else {
            draw_header(&mut buffer, app, area);
            let body = Rect::new(0, 2, width, height.saturating_sub(4));
            match app.screen() {
                Screen::GameSelect => draw_game_select(&mut buffer, app, body),
                Screen::StageSelect => draw_stage_select(&mut buffer, app, body),
                Screen::Ready => draw_ready(&mut buffer, app, body),
                Screen::Playing => draw_playing(&mut buffer, app, body),
                Screen::Result => draw_result(&mut buffer, app, body),
            }
            draw_footer(&mut buffer, app, area);
        }
        self.present(output, buffer)
    }

    fn present(&mut self, output: &mut impl Write, current: Buffer) -> io::Result<()> {
        let full = self
            .previous
            .as_ref()
            .map(|previous| previous.width != current.width || previous.height != current.height)
            .unwrap_or(true);
        if full {
            queue!(output, Clear(ClearType::All))?;
        }

        for y in 0..current.height {
            for x in 0..current.width {
                let index = current.index(x, y);
                let cell = current.cells[index];
                if cell.character == '\0' {
                    continue;
                }
                if !full
                    && self
                        .previous
                        .as_ref()
                        .and_then(|previous| previous.cells.get(index))
                        == Some(&cell)
                {
                    continue;
                }
                queue!(
                    output,
                    MoveTo(x, y),
                    SetAttribute(Attribute::Reset),
                    SetForegroundColor(cell.paint.foreground),
                    SetBackgroundColor(cell.paint.background)
                )?;
                if cell.paint.bold {
                    queue!(output, SetAttribute(Attribute::Bold))?;
                }
                queue!(output, Print(cell.character))?;
            }
        }
        queue!(output, ResetColor, SetAttribute(Attribute::Reset))?;
        output.flush()?;
        self.previous = Some(current);
        Ok(())
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

fn draw_small(buffer: &mut Buffer, area: Rect) {
    let panel = centered(
        area,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );
    buffer.panel(panel, " ARCADE / RESIZE ", BORDER_BRIGHT, PANEL);
    let inner = panel.inset(2);
    buffer.centered_text(
        inner,
        inner.y.saturating_add(2),
        "터미널 창을 조금 더 넓혀 주세요",
        Paint::new(TEXT, PANEL, true),
    );
    buffer.centered_text(
        inner,
        inner.y.saturating_add(4),
        "권장 크기  64 × 24 이상",
        Paint::new(MUTED, PANEL, false),
    );
}

fn draw_header(buffer: &mut Buffer, app: &App, area: Rect) {
    let header = Rect::new(area.x, area.y, area.width, 2.min(area.height));
    buffer.fill(header, Paint::new(TEXT, HEADER_BG, false));
    buffer.line(0, 1, area.width, '─', Paint::new(BORDER, HEADER_BG, false));
    buffer.text(1, 0, "◈ ARCADE", Paint::new(TEXT, HEADER_BG, true));
    let title = match app.screen() {
        Screen::GameSelect => "ARCADE FLOOR",
        Screen::StageSelect => "CABINET / MODE SELECT",
        Screen::Ready => "INSERT COIN",
        Screen::Playing => "NOW PLAYING",
        Screen::Result => "GAME OVER",
    };
    let game = app
        .selected_game_definition()
        .map(|definition| definition.display_name.as_str())
        .unwrap_or("아케이드");
    let center = Rect::new(18, 0, area.width.saturating_sub(40), 1);
    buffer.centered_text(
        center,
        0,
        &format!("{}  /  {}", title, game),
        Paint::new(CYAN, HEADER_BG, true),
    );
    let wallet = format!("◉ {:>6} WON", app.wallet_won());
    let wallet_x = area.width.saturating_sub(wallet.width() as u16 + 2);
    buffer.text(wallet_x, 0, &wallet, Paint::new(YELLOW, HEADER_BG, true));
}

fn draw_footer(buffer: &mut Buffer, app: &App, area: Rect) {
    let y = area.height.saturating_sub(2);
    buffer.line(0, y, area.width, '─', Paint::new(BORDER, BG, false));
    let help = match app.screen() {
        Screen::GameSelect => "↑↓ SELECT CABINET   ENTER INSERT COIN   ESC EXIT",
        Screen::StageSelect => "↑↓ SELECT MODE   ENTER READY   ESC BACK",
        Screen::Ready => "ENTER OPEN TABLE / START GAME   ESC BACK   Ctrl+C EXIT",
        Screen::Playing if app.gambling_feedback().is_some() => {
            "ENTER NEXT HAND   ESC EXIT CABINET   Ctrl+C QUIT"
        }
        Screen::Playing if app.snake_session().is_some() => {
            "ARROWS/WASD MOVE   ESC EXIT CABINET   Ctrl+C QUIT"
        }
        Screen::Playing if app.tictactoe_session().is_some() => {
            "ARROWS SELECT   ENTER PLACE   ESC EXIT   Ctrl+C QUIT"
        }
        Screen::Playing if app.game2048_session().is_some() => {
            "ARROWS/WASD MOVE   ESC EXIT   Ctrl+C QUIT"
        }
        Screen::Playing if app.sudoku_session().is_some() => {
            "ARROWS MOVE   NUMBER INPUT   BACKSPACE CLEAR   ESC EXIT"
        }
        Screen::Playing if app.blackjack_session().is_some() => {
            "TYPE BET   ENTER/H HIT   S/SPACE STAND   ESC EXIT   Ctrl+C QUIT"
        }
        Screen::Playing if app.roulette_session().is_some() => {
            "TYPE BET   R RED   B BLACK   G GREEN   ENTER SPIN   ESC EXIT"
        }
        Screen::Playing if app.slots_session().is_some() => {
            "TYPE BET   ENTER/SPACE PULL   ESC EXIT   Ctrl+C QUIT"
        }
        Screen::Playing if app.holdem_session().is_some() => {
            "TYPE BET   C CHECK   R +10   T +25   Y +50   F FOLD   ESC EXIT"
        }
        Screen::Playing if app.typing_session().is_some() => {
            "TYPE   ENTER SUBMIT   BACKSPACE EDIT   ESC EXIT"
        }
        Screen::Playing if app.breakout_session().is_some() => {
            "ARROWS/A-D MOVE PADDLE   ESC EXIT   Ctrl+C QUIT"
        }
        Screen::Playing if app.minesweeper_session().is_some() => {
            "ARROWS/WASD MOVE   ENTER REVEAL   F FLAG   ESC EXIT"
        }
        Screen::Playing if app.connect_four_session().is_some() => {
            "LEFT/RIGHT SELECT   ENTER DROP   ESC EXIT"
        }
        Screen::Playing if app.memory_match_session().is_some() => {
            "ARROWS/WASD SELECT   ENTER FLIP/CONTINUE   ESC EXIT"
        }
        Screen::Playing if app.maze_session().is_some() => "ARROWS/WASD EXPLORE   ESC EXIT",
        Screen::Playing => "NUMBER INPUT   BACKSPACE EDIT   ENTER SUBMIT   ESC EXIT",
        Screen::Result => "ENTER/ESC ARCADE FLOOR   Q EXIT",
    };
    buffer.text(2, y.saturating_add(1), help, Paint::new(MUTED, BG, false));
}

fn draw_game_select(buffer: &mut Buffer, app: &App, area: Rect) {
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.panel(panel, " ARCADE FLOOR ", BORDER_BRIGHT, SURFACE);
    let inner = panel.inset(2);
    buffer.text(
        inner.x,
        inner.y,
        "INSERT COIN // CHOOSE A CABINET",
        Paint::new(TEXT, SURFACE, true),
    );
    buffer.text(
        inner.x,
        inner.y.saturating_add(1),
        "게임을 고르면 그 게임의 세계로 입장합니다",
        Paint::new(MUTED, SURFACE, false),
    );
    let content = Rect::new(
        inner.x,
        inner.y.saturating_add(3),
        inner.width,
        inner.height.saturating_sub(3),
    );
    if content.width >= 68 {
        let list = Rect::new(content.x, content.y, 42, content.height);
        let detail = Rect::new(
            content.x.saturating_add(44),
            content.y,
            content.width.saturating_sub(44),
            content.height,
        );
        draw_cabinet_list(buffer, app, list);
        draw_game_preview(buffer, app, detail);
    } else {
        draw_cabinet_list(buffer, app, content);
    }
}

fn draw_cabinet_list(buffer: &mut Buffer, app: &App, rect: Rect) {
    buffer.panel(rect, " CABINET ROW ", BORDER, PANEL);
    let games = app.catalog().games();
    let visible_rows = usize::from(rect.height.saturating_sub(3)).max(1);
    let start = cabinet_window_start(app.selected_game_index(), games.len(), visible_rows);
    for (row_index, (index, game)) in games
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_rows)
        .enumerate()
    {
        let y = rect.y.saturating_add(2 + row_index as u16);
        let selected = index == app.selected_game_index();
        let row = Rect::new(rect.x.saturating_add(1), y, rect.width.saturating_sub(2), 1);
        if selected {
            buffer.fill(row, Paint::new(TEXT, PANEL_HOVER, true));
        }
        let background = if selected { PANEL_HOVER } else { PANEL };
        let foreground = if selected { TEXT } else { MUTED };
        let marker = if selected { "▶" } else { " " };
        buffer.text(row.x, y, marker, Paint::new(CYAN, background, true));
        buffer.text(
            row.x.saturating_add(2),
            y,
            game_icon(Some(game.kind)),
            Paint::new(game_color(game.kind), background, true),
        );
        buffer.text(
            row.x.saturating_add(5),
            y,
            &format!("{:02}", index + 1),
            Paint::new(MUTED, background, false),
        );
        buffer.text(
            row.x.saturating_add(9),
            y,
            &game.display_name,
            Paint::new(foreground, background, true),
        );
        let stages = format!("{}", game.stages.len());
        let stage_x = row.x.saturating_add(row.width.saturating_sub(3));
        buffer.text(
            stage_x,
            y,
            &stages,
            Paint::new(game_color(game.kind), background, true),
        );
    }
    if start > 0 {
        buffer.text(
            rect.x.saturating_add(rect.width.saturating_sub(3)),
            rect.y.saturating_add(1),
            "▲",
            Paint::new(CYAN, PANEL, true),
        );
    }
    if start + visible_rows < games.len() {
        buffer.text(
            rect.x.saturating_add(rect.width.saturating_sub(3)),
            rect.y.saturating_add(rect.height.saturating_sub(1)),
            "▼",
            Paint::new(CYAN, PANEL, true),
        );
    }
}

fn cabinet_window_start(selected: usize, total: usize, visible: usize) -> usize {
    let visible = visible.max(1);
    selected
        .saturating_sub(visible / 2)
        .min(total.saturating_sub(visible))
}

fn draw_game_preview(buffer: &mut Buffer, app: &App, rect: Rect) {
    buffer.panel(rect, " CABINET PREVIEW ", BORDER, PANEL);
    let Some(game) = app.selected_game_definition() else {
        return;
    };
    let inner = rect.inset(2);
    let accent = game_color(game.kind);
    buffer.centered_text(
        inner,
        inner.y.saturating_add(1),
        game_icon(Some(game.kind)),
        Paint::new(accent, PANEL, true),
    );
    buffer.centered_text(
        inner,
        inner.y.saturating_add(3),
        &game.display_name,
        Paint::new(TEXT, PANEL, true),
    );
    buffer.centered_text(
        inner,
        inner.y.saturating_add(4),
        "PRESS START",
        Paint::new(GREEN, PANEL, true),
    );
    wrapped_text(
        buffer,
        inner,
        inner.y.saturating_add(6),
        game_summary(game),
        Paint::new(MUTED, PANEL, false),
    );
    buffer.text(
        inner.x,
        inner.y.saturating_add(inner.height.saturating_sub(3)),
        &format!("{} modes", game.stages.len()),
        Paint::new(BLUE, PANEL, false),
    );
    buffer.text(
        inner.x,
        inner.y.saturating_add(inner.height.saturating_sub(2)),
        "ENTER  INSERT COIN",
        Paint::new(BG, CYAN, true),
    );
}

fn draw_stage_select(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(game) = app.selected_game_definition() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.panel(
        panel,
        &format!(" {} / MODE SELECT ", game.display_name),
        game_color(game.kind),
        SURFACE,
    );
    let inner = panel.inset(2);
    buffer.text(
        inner.x,
        inner.y,
        "SELECT YOUR MODE",
        Paint::new(TEXT, SURFACE, true),
    );
    buffer.text(
        inner.x,
        inner.y.saturating_add(1),
        "규칙과 난이도를 고른 뒤 캐비닛에 코인을 넣습니다",
        Paint::new(MUTED, SURFACE, false),
    );
    let content = Rect::new(
        inner.x,
        inner.y.saturating_add(3),
        inner.width,
        inner.height.saturating_sub(3),
    );
    if content.width >= 70 {
        let list = Rect::new(content.x, content.y, 45, content.height);
        let detail = Rect::new(
            content.x.saturating_add(47),
            content.y,
            content.width.saturating_sub(47),
            content.height,
        );
        draw_stage_list(buffer, app, game, list);
        draw_stage_preview(buffer, app, game, detail);
    } else {
        draw_stage_list(buffer, app, game, content);
    }
}

fn draw_stage_list(buffer: &mut Buffer, app: &App, game: &GameDefinition, rect: Rect) {
    buffer.panel(rect, " GAME MODE ", BORDER, PANEL);
    for (index, stage) in game.stages.iter().enumerate() {
        let y = rect.y.saturating_add(2 + index as u16 * 2);
        if y.saturating_add(1) >= rect.y.saturating_add(rect.height).saturating_sub(1) {
            break;
        }
        let selected = index == app.selected_stage_index();
        let background = if selected { PANEL_HOVER } else { PANEL };
        if selected {
            buffer.fill(
                Rect::new(rect.x.saturating_add(1), y, rect.width.saturating_sub(2), 2),
                Paint::new(TEXT, background, true),
            );
        }
        buffer.text(
            rect.x.saturating_add(2),
            y,
            if selected { "▶" } else { " " },
            Paint::new(CYAN, background, true),
        );
        buffer.text(
            rect.x.saturating_add(5),
            y,
            stage_badge(stage.difficulty_order),
            Paint::new(game_color(stage.game_kind), background, true),
        );
        buffer.text(
            rect.x.saturating_add(10),
            y,
            &stage.display_name,
            Paint::new(if selected { TEXT } else { MUTED }, background, true),
        );
        buffer.text(
            rect.x.saturating_add(10),
            y.saturating_add(1),
            &format!("{}  ·  {}", stage.id, stage_time_limit(stage)),
            Paint::new(MUTED, background, false),
        );
    }
}

fn draw_stage_preview(buffer: &mut Buffer, app: &App, game: &GameDefinition, rect: Rect) {
    buffer.panel(rect, " MODE PREVIEW ", BORDER, PANEL);
    let Some(stage) = app.selected_stage_definition() else {
        return;
    };
    let inner = rect.inset(2);
    buffer.text(
        inner.x,
        inner.y,
        &stage.display_name,
        Paint::new(TEXT, PANEL, true),
    );
    wrapped_text(
        buffer,
        inner,
        inner.y.saturating_add(2),
        stage_description(stage.game_kind, stage),
        Paint::new(MUTED, PANEL, false),
    );
    buffer.text(
        inner.x,
        inner.y.saturating_add(6),
        &format!("DIFFICULTY  LEVEL {}", stage.difficulty_order),
        Paint::new(YELLOW, PANEL, false),
    );
    buffer.text(
        inner.x,
        inner.y.saturating_add(7),
        &format!("TIME LIMIT  {}", stage_time_limit(stage)),
        Paint::new(TEXT, PANEL, false),
    );
    let best = app
        .best_record(&game.id, &stage.id)
        .map(|record| format!("BEST SCORE  {} pts", record.score))
        .unwrap_or_else(|| "BEST SCORE  —".to_string());
    buffer.text(
        inner.x,
        inner.y.saturating_add(8),
        &best,
        Paint::new(YELLOW, PANEL, false),
    );
    buffer.text(
        inner.x,
        inner.y.saturating_add(inner.height.saturating_sub(2)),
        "ENTER  PRESS START",
        Paint::new(BG, CYAN, true),
    );
}

fn draw_ready(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(game) = app.selected_game_definition() else {
        return;
    };
    let Some(stage) = app.selected_stage_definition() else {
        return;
    };
    let panel = centered(area, 82.min(area.width), 16.min(area.height));
    buffer.panel(
        panel,
        &format!(" {} / PRESS START ", game.display_name),
        CYAN,
        SURFACE,
    );
    let inner = panel.inset(2);
    let accent = game_color(game.kind);
    buffer.centered_text(
        inner,
        inner.y.saturating_add(1),
        game_icon(Some(game.kind)),
        Paint::new(accent, SURFACE, true),
    );
    buffer.centered_text(
        inner,
        inner.y.saturating_add(3),
        &stage.display_name,
        Paint::new(TEXT, SURFACE, true),
    );
    buffer.centered_text(
        inner,
        inner.y.saturating_add(5),
        stage_description(stage.game_kind, stage),
        Paint::new(MUTED, SURFACE, false),
    );
    buffer.centered_text(
        inner,
        inner.y.saturating_add(7),
        game_controls(stage.game_kind),
        Paint::new(TEXT, SURFACE, false),
    );
    let is_gambling = matches!(
        stage.game_kind,
        GameKind::Blackjack | GameKind::Roulette | GameKind::Slots | GameKind::Holdem
    );
    if is_gambling {
        buffer.centered_text(
            inner,
            inner.y.saturating_add(9),
            "TABLE SETTINGS  ·  BET INSIDE GAME",
            Paint::new(YELLOW, SURFACE, true),
        );
        buffer.centered_text(
            inner,
            inner.y.saturating_add(10),
            &format!("WALLET  {} WON", app.wallet_won()),
            Paint::new(MUTED, SURFACE, false),
        );
        buffer.centered_text(
            inner,
            inner.y.saturating_add(11),
            "ENTER OPEN TABLE   ·   SET WAGER AFTER ENTRY",
            Paint::new(BG, CYAN, true),
        );
        if let Some(message) = app.status_message() {
            buffer.centered_text(
                inner,
                inner.y.saturating_add(12),
                message,
                Paint::new(RED, SURFACE, false),
            );
        }
    } else {
        let best = app
            .best_record(&game.id, &stage.id)
            .map(|record| format!("HIGH SCORE  {} pts", record.score))
            .unwrap_or_else(|| "HIGH SCORE  —".to_string());
        buffer.centered_text(
            inner,
            inner.y.saturating_add(9),
            &best,
            Paint::new(YELLOW, SURFACE, false),
        );
        buffer.centered_text(
            inner,
            inner.y.saturating_add(11),
            "ENTER  START GAME",
            Paint::new(BG, CYAN, true),
        );
    }
}

fn draw_playing(buffer: &mut Buffer, app: &App, area: Rect) {
    if app.snake_session().is_some() {
        draw_snake(buffer, app, area);
    } else if app.tictactoe_session().is_some() {
        draw_tictactoe(buffer, app, area);
    } else if app.game2048_session().is_some() {
        draw_2048(buffer, app, area);
    } else if app.sudoku_session().is_some() {
        draw_sudoku(buffer, app, area);
    } else if app.blackjack_session().is_some() {
        draw_blackjack(buffer, app, area);
    } else if app.roulette_session().is_some() {
        draw_roulette(buffer, app, area);
    } else if app.slots_session().is_some() {
        draw_slots(buffer, app, area);
    } else if app.holdem_session().is_some() {
        draw_holdem(buffer, app, area);
    } else if app.typing_session().is_some() {
        draw_typing(buffer, app, area);
    } else if app.breakout_session().is_some() {
        draw_breakout(buffer, app, area);
    } else if app.minesweeper_session().is_some() {
        draw_minesweeper(buffer, app, area);
    } else if app.connect_four_session().is_some() {
        draw_connect_four(buffer, app, area);
    } else if app.memory_match_session().is_some() {
        draw_memory_match(buffer, app, area);
    } else if app.maze_session().is_some() {
        draw_maze(buffer, app, area);
    } else {
        draw_math(buffer, app, area);
    }
}

fn draw_math(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.round_session() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.panel(panel, " MATH RUSH ", BLUE, SURFACE);
    let inner = panel.inset(2);
    let hud_width = if inner.width >= 70 { 25 } else { 0 };
    let main = Rect::new(
        inner.x,
        inner.y,
        inner
            .width
            .saturating_sub(hud_width + u16::from(hud_width > 0)),
        inner.height,
    );
    draw_math_board(buffer, session, app, main);
    if hud_width > 0 {
        let hud = Rect::new(
            main.x.saturating_add(main.width + 1),
            inner.y,
            hud_width,
            inner.height,
        );
        draw_math_hud(buffer, session, hud);
    }
}

fn draw_math_board(buffer: &mut Buffer, session: &RoundSession, app: &App, rect: Rect) {
    buffer.panel(rect, " QUESTION ", BORDER, PANEL);
    let inner = rect.inset(2);
    buffer.centered_text(
        inner,
        inner.y.saturating_add(3),
        &session.current_problem().equation(),
        Paint::new(BLUE, PANEL, true),
    );
    let answer = if session.answer_input().is_empty() {
        "_"
    } else {
        session.answer_input()
    };
    buffer.centered_text(
        inner,
        inner.y.saturating_add(6),
        &format!("ANSWER  {}", answer),
        Paint::new(TEXT, PANEL_ALT, true),
    );
    let feedback = match session.last_answer_correct() {
        Some(true) => "CORRECT!",
        Some(false) => "TRY THE NEXT ONE",
        None => "SOLVE THE EQUATION",
    };
    buffer.centered_text(
        inner,
        inner.y.saturating_add(9),
        feedback,
        Paint::new(
            if session.last_answer_correct() == Some(true) {
                GREEN
            } else {
                MUTED
            },
            PANEL,
            true,
        ),
    );
    if let Some(message) = app.status_message() {
        buffer.centered_text(
            inner,
            inner.y.saturating_add(11),
            message,
            Paint::new(RED, PANEL, false),
        );
    }
}

fn draw_math_hud(buffer: &mut Buffer, session: &RoundSession, rect: Rect) {
    buffer.panel(rect, " HUD ", BORDER, PANEL);
    let inner = rect.inset(2);
    let ratio = (session.remaining().as_secs_f64() / session.time_limit().as_secs_f64().max(1.0))
        .clamp(0.0, 1.0);
    buffer.text(inner.x, inner.y, "TIME", Paint::new(MUTED, PANEL, true));
    draw_bar(
        buffer,
        Rect::new(inner.x, inner.y.saturating_add(1), inner.width, 1),
        ratio,
        if ratio < 0.25 { RED } else { BLUE },
        GRID,
    );
    buffer.centered_text(
        inner,
        inner.y.saturating_add(2),
        &format!("{}s", session.remaining_seconds()),
        Paint::new(TEXT, PANEL, true),
    );
    let rows = [
        ("CORRECT", session.correct_answers().to_string(), GREEN),
        ("ATTEMPTS", session.attempts().to_string(), TEXT),
        ("STREAK", session.current_streak().to_string(), YELLOW),
        ("BEST", session.best_streak().to_string(), PURPLE),
    ];
    for (index, (label, value, color)) in rows.iter().enumerate() {
        let y = inner.y.saturating_add(5 + index as u16 * 2);
        buffer.text(inner.x, y, label, Paint::new(MUTED, PANEL, false));
        buffer.text(
            inner
                .x
                .saturating_add(inner.width.saturating_sub(value.width() as u16)),
            y,
            value,
            Paint::new(*color, PANEL, true),
        );
    }
}

fn draw_snake(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.snake_session() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.panel(panel, " SNAKE // SURVIVAL ", GREEN, SURFACE);
    let inner = panel.inset(2);
    let cell_width = if inner.width >= 58 { 2 } else { 1 };
    let board_width = BOARD_WIDTH as u16 * cell_width + 2;
    let board = Rect::new(inner.x, inner.y, board_width, 16.min(inner.height));
    buffer.panel(board, " BOARD ", BORDER, PANEL);
    let board_inner = board.inset(1);
    let head = session.snake().first().copied();
    let food = session.food();
    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            let point = Point { x, y };
            let px = board_inner.x.saturating_add(x as u16 * cell_width);
            let py = board_inner.y.saturating_add(y as u16);
            let is_head = head == Some(point);
            let is_body = session.snake().contains(&point);
            let is_food = food == point;
            let (glyph, paint) = if is_head {
                (
                    if cell_width == 2 { "██" } else { "█" },
                    Paint::new(BG, GREEN, true),
                )
            } else if is_body {
                (
                    if cell_width == 2 { "▓▓" } else { "▓" },
                    Paint::new(GREEN, GRID, true),
                )
            } else if is_food {
                (
                    if cell_width == 2 { "✦ " } else { "✦" },
                    Paint::new(RED, PANEL, true),
                )
            } else {
                (
                    if cell_width == 2 { "· " } else { "·" },
                    Paint::new(GRID, PANEL, false),
                )
            };
            buffer.text(px, py, glyph, paint);
        }
    }
    let hud = Rect::new(
        board.x.saturating_add(board.width + 2),
        inner.y,
        inner.width.saturating_sub(board.width + 2),
        board.height,
    );
    draw_simple_hud(
        buffer,
        hud,
        " HUD ",
        &[
            ("PACE", session.pace().label().to_string(), CYAN),
            ("SCORE", session.score().to_string(), YELLOW),
            ("LENGTH", session.snake().len().to_string(), GREEN),
            ("FOOD", "✦".to_string(), RED),
        ],
        &format!("DIRECTION  {}", direction_label(session.direction())),
    );
}

fn draw_2048(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.game2048_session() else {
        return;
    };
    let panel = centered(area, area.width.saturating_sub(2), area.height);
    buffer.panel(panel, " 2048 // MERGE THE FUTURE ", YELLOW, SURFACE);
    let inner = panel.inset(1);
    let tile_width = if inner.width >= 50 { 11 } else { 9 };
    let tile_height = if inner.height >= 22 { 4 } else { 3 };
    let board_width = GRID_SIZE as u16 * tile_width + (GRID_SIZE as u16 - 1) + 2;
    let board_height = GRID_SIZE as u16 * tile_height + (GRID_SIZE as u16 - 1) + 2;
    draw_2048_status(buffer, app, session, inner);
    let board = Rect::new(
        inner.x + inner.width.saturating_sub(board_width) / 2,
        inner.y.saturating_add(1),
        board_width,
        board_height,
    );
    buffer.panel(board, " GRID // 4 × 4 ", BORDER_BRIGHT, PANEL_ALT);
    let cells = board.inset(1);
    let frame = (session.elapsed().as_millis() / 220) as usize;
    for (row_index, row) in session.board().iter().enumerate() {
        for (column_index, tile) in row.iter().enumerate() {
            let x = cells.x + column_index as u16 * (tile_width + 1);
            let y = cells.y + row_index as u16 * (tile_height + 1);
            let shimmer = (frame + row_index * GRID_SIZE + column_index) % 13 == 0;
            draw_2048_tile(
                buffer,
                Rect::new(x, y, tile_width, tile_height),
                *tile,
                shimmer,
            );
        }
    }
}

fn draw_2048_tile(buffer: &mut Buffer, rect: Rect, tile: u32, shimmer: bool) {
    let (foreground, background) = tile_colors(tile);
    let border = tile_border(tile, shimmer);
    let highlight = tile_highlight(tile, shimmer);
    let shadow = tile_shadow(tile);
    buffer.fill(rect, Paint::new(background, background, false));

    for x in rect.x..rect.x.saturating_add(rect.width) {
        buffer.set(x, rect.y, '▀', Paint::new(highlight, background, true));
        buffer.set(
            x,
            rect.y.saturating_add(rect.height.saturating_sub(1)),
            '▄',
            Paint::new(shadow, background, false),
        );
    }
    for y in rect.y.saturating_add(1)..rect.y.saturating_add(rect.height.saturating_sub(1)) {
        buffer.set(rect.x, y, '▌', Paint::new(highlight, background, true));
        buffer.set(
            rect.x.saturating_add(rect.width.saturating_sub(1)),
            y,
            '▐',
            Paint::new(shadow, background, false),
        );
    }

    if tile == 0 {
        if shimmer {
            buffer.centered_text(
                rect,
                rect.y + rect.height / 2,
                "·  ·",
                Paint::new(border, background, true),
            );
        }
    } else {
        buffer.centered_text(
            rect,
            rect.y + rect.height / 2,
            &tile.to_string(),
            Paint::new(foreground, background, true),
        );
    }
}

fn draw_2048_status(buffer: &mut Buffer, app: &App, session: &Game2048Session, rect: Rect) {
    let best_score = if let (Some(game), Some(stage)) = (
        app.selected_game_definition(),
        app.selected_stage_definition(),
    ) {
        app.best_record(&game.id, &stage.id)
            .map(|record| record.score)
            .unwrap_or(0)
    } else {
        0
    };
    let elapsed = session.elapsed().as_secs();
    let run_time = format!("{:02}:{:02}", elapsed / 60, elapsed % 60);
    let max_tile = session.board().iter().flatten().copied().max().unwrap_or(0);
    let left = format!("◈ SCORE {:05}", session.score());
    let center = format!("MOVES {:02}   TIME {run_time}", session.moves());
    let right = format!("BEST {:05}  ◇ {}", best_score, max_tile.max(2));

    buffer.text(rect.x, rect.y, &left, Paint::new(YELLOW, SURFACE, true));
    buffer.centered_text(rect, rect.y, &center, Paint::new(TEXT, SURFACE, false));
    let right_x = rect
        .x
        .saturating_add(rect.width.saturating_sub(right.width() as u16));
    buffer.text(right_x, rect.y, &right, Paint::new(CYAN, SURFACE, true));

    if session.is_game_over() {
        buffer.centered_text(
            rect,
            rect.y,
            "▣ GAME OVER ▣",
            Paint::new(RED, SURFACE, true),
        );
    }
}

fn draw_sudoku(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.sudoku_session() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.panel(panel, " SUDOKU // NINE LIVES ", PURPLE, SURFACE);
    let inner = panel.inset(2);
    let cell_width = 4;
    let cell_height = if inner.height >= 22 { 2 } else { 1 };
    let board = Rect::new(
        inner.x,
        inner.y,
        SUDOKU_SIZE as u16 * cell_width + 2,
        SUDOKU_SIZE as u16 * cell_height + 2,
    );
    buffer.panel(board, " GRID ", BORDER, PANEL);
    let cells = board.inset(1);
    for row in 0..SUDOKU_SIZE {
        for column in 0..SUDOKU_SIZE {
            let index = row * SUDOKU_SIZE + column;
            let x = cells.x + column as u16 * cell_width;
            let y = cells.y + row as u16 * cell_height;
            let selected = index == session.cursor();
            let background = if selected { PANEL_HOVER } else { PANEL };
            let value = session.board()[index];
            let label = if value == 0 {
                "·".to_string()
            } else {
                value.to_string()
            };
            let foreground = if session.is_wrong(index) || session.is_conflict(index) {
                RED
            } else if session.is_given(index) {
                TEXT
            } else if value != 0 {
                CYAN
            } else {
                MUTED
            };
            buffer.fill(
                Rect::new(x, y, cell_width.saturating_sub(1), cell_height),
                Paint::new(foreground, background, session.is_given(index)),
            );
            buffer.centered_text(
                Rect::new(x, y, cell_width.saturating_sub(1), cell_height),
                y,
                &label,
                Paint::new(foreground, background, session.is_given(index)),
            );
        }
    }
    let hud = Rect::new(
        board.x.saturating_add(board.width + 2),
        inner.y,
        inner.width.saturating_sub(board.width + 2),
        board.height,
    );
    draw_simple_hud(
        buffer,
        hud,
        " HUD ",
        &[
            ("MODE", session.difficulty().label().to_string(), PURPLE),
            ("FILLED", format!("{}/81", session.filled_count()), CYAN),
            ("MISTAKES", session.mistakes().to_string(), RED),
        ],
        "ARROWS MOVE  1-9 FILL",
    );
}

fn draw_tictactoe(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.tictactoe_session() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.panel(panel, " TIC-TAC-TOE // DUEL ", CYAN, SURFACE);
    let inner = panel.inset(2);
    let board = Rect::new(inner.x, inner.y, 35, 15);
    buffer.panel(board, " BOARD ", BORDER, PANEL);
    let cells = board.inset(1);
    for row in 0..3 {
        for column in 0..3 {
            let index = row * 3 + column;
            let x = cells.x + column as u16 * 11;
            let y = cells.y + row as u16 * 4;
            let selected = session.cursor() == index;
            let background = if selected { PANEL_HOVER } else { PANEL };
            buffer.fill(Rect::new(x, y, 9, 3), Paint::new(TEXT, background, false));
            let (symbol, color) = match session.board()[index] {
                TicTacToeCell::Empty => ("·", MUTED),
                TicTacToeCell::X => ("X", CYAN),
                TicTacToeCell::O => ("O", YELLOW),
            };
            buffer.centered_text(
                Rect::new(x, y, 9, 3),
                y.saturating_add(1),
                symbol,
                Paint::new(color, background, true),
            );
        }
    }
    let winner = match session.winner() {
        Some(TicTacToeCell::X) => "YOU WIN",
        Some(TicTacToeCell::O) => "CPU WINS",
        _ if session.is_game_over() => "DRAW",
        _ => "YOUR TURN",
    };
    let hud = Rect::new(
        board.x.saturating_add(board.width + 2),
        inner.y,
        inner.width.saturating_sub(board.width + 2),
        board.height,
    );
    draw_simple_hud(
        buffer,
        hud,
        " DUEL HUD ",
        &[
            ("STATUS", winner.to_string(), GREEN),
            ("YOU", "X".to_string(), CYAN),
            ("CPU", "O".to_string(), YELLOW),
        ],
        app.status_message().unwrap_or("ARROWS SELECT  ENTER PLACE"),
    );
}

fn draw_blackjack(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.blackjack_session() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.fill(
        panel,
        Paint::new(TEXT, Color::Rgb { r: 8, g: 53, b: 45 }, false),
    );
    buffer.panel(
        panel,
        " BLACKJACK // TABLE 21 ",
        YELLOW,
        Color::Rgb { r: 8, g: 53, b: 45 },
    );
    let inner = panel.inset(2);
    buffer.text(
        inner.x.saturating_add(inner.width.saturating_sub(16)),
        inner.y,
        &gambling_wager_label(app),
        Paint::new(YELLOW, Color::Rgb { r: 8, g: 53, b: 45 }, true),
    );
    buffer.text(
        inner.x,
        inner.y,
        "DEALER",
        Paint::new(YELLOW, Color::Rgb { r: 8, g: 53, b: 45 }, true),
    );
    if app.card_hand_started() {
        draw_cards(
            buffer,
            session.dealer_cards(),
            inner.x,
            inner.y + 2,
            !session.dealer_revealed(),
        );
    } else {
        draw_card_backs(buffer, inner.x, inner.y + 2, 2);
    }
    buffer.text(
        inner.x,
        inner.y.saturating_add(7),
        &if app.card_hand_started() {
            format!(
                "VISIBLE SCORE  {}",
                if session.dealer_revealed() {
                    session.dealer_score()
                } else {
                    session.dealer_visible_score()
                }
            )
        } else {
            "VISIBLE SCORE  --".to_string()
        },
        Paint::new(TEXT, Color::Rgb { r: 8, g: 53, b: 45 }, false),
    );
    buffer.text(
        inner.x,
        inner.y.saturating_add(9),
        "PLAYER",
        Paint::new(CYAN, Color::Rgb { r: 8, g: 53, b: 45 }, true),
    );
    if app.card_hand_started() {
        draw_cards(buffer, session.player_cards(), inner.x, inner.y + 11, false);
    } else {
        draw_card_backs(buffer, inner.x, inner.y + 11, 2);
    }
    buffer.text(
        inner.x,
        inner.y.saturating_add(16),
        &if app.card_hand_started() {
            format!("SCORE  {}", session.player_score())
        } else {
            "SCORE  --".to_string()
        },
        Paint::new(TEXT, Color::Rgb { r: 8, g: 53, b: 45 }, true),
    );
    let feedback = gambling_feedback_line(app)
        .or_else(|| app.status_message().map(str::to_string))
        .unwrap_or_else(|| {
            if !app.card_hand_betting_open() {
                "ENTER START HAND   ·   ESC LEAVE TABLE".to_string()
            } else if !app.card_hand_started() {
                "TYPE BET AMOUNT   ·   ENTER DEAL CARDS".to_string()
            } else {
                "ENTER/H HIT   S/SPACE STAND".to_string()
            }
        });
    buffer.text(
        inner.x,
        inner.y.saturating_add(inner.height.saturating_sub(2)),
        &feedback,
        Paint::new(
            if session.is_game_over() { GREEN } else { MUTED },
            Color::Rgb { r: 8, g: 53, b: 45 },
            true,
        ),
    );
}

fn draw_holdem(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.holdem_session() else {
        return;
    };
    let felt = Color::Rgb { r: 7, g: 68, b: 51 };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.fill(panel, Paint::new(TEXT, felt, false));
    buffer.panel(panel, " TEXAS HOLD'EM // AI TABLE ", YELLOW, felt);
    let inner = panel.inset(2);
    let difficulty = session.difficulty().label();
    let hand_started = app.card_hand_started();
    buffer.text(
        inner.x,
        inner.y,
        &format!("OPPONENT  AI // {difficulty}"),
        Paint::new(YELLOW, felt, true),
    );
    buffer.text(
        inner.x.saturating_add(inner.width.saturating_sub(25)),
        inner.y,
        &format!(
            "{}  POT {:03}  {}",
            session.street().label(),
            session.pot(),
            gambling_wager_label(app)
        ),
        Paint::new(TEXT, felt, false),
    );
    draw_holdem_card_row(
        buffer,
        inner,
        session.opponent_cards(),
        !hand_started || session.community_cards().len() < 5,
        inner.y.saturating_add(1),
        2,
    );

    buffer.text(
        inner.x,
        inner.y.saturating_add(5),
        "COMMUNITY CARDS // FLOP · TURN · RIVER",
        Paint::new(CYAN, felt, true),
    );
    draw_holdem_card_row(
        buffer,
        inner,
        session.community_cards(),
        !hand_started,
        inner.y.saturating_add(6),
        5,
    );

    buffer.text(
        inner.x,
        inner.y.saturating_add(10),
        "YOUR HAND",
        Paint::new(CYAN, felt, true),
    );
    draw_holdem_card_row(
        buffer,
        inner,
        session.player_cards(),
        !hand_started,
        inner.y.saturating_add(11),
        2,
    );

    let action = session
        .last_ai_action()
        .map(HoldemAiAction::label)
        .unwrap_or("대기");
    let status = if let Some(outcome) = session.outcome() {
        let player_hand = session.player_hand_name().unwrap_or("미공개");
        let opponent_hand = session.opponent_hand_name().unwrap_or("미공개");
        format!(
            "{}  · YOU {} / AI {}  · {} · +{} WON",
            outcome.label(),
            player_hand,
            opponent_hand,
            session.difficulty().label(),
            session.payout().saturating_mul(app.gambling_total_wager())
        )
    } else if let Some(message) = app.status_message() {
        message.to_string()
    } else if !app.card_hand_betting_open() {
        "ENTER START HAND   ·   ESC LEAVE TABLE".to_string()
    } else if !hand_started {
        "TYPE BET AMOUNT   ·   ENTER DEAL CARDS".to_string()
    } else {
        let controls = if app.gambling_bet_committed() {
            "C CHECK/CALL   R +10   T +25   Y +50   F FOLD"
        } else {
            "TYPE BET AMOUNT   ·   C CHECK/CALL   R +10   T +25   Y +50   F FOLD"
        };
        format!("AI ACTION  {action}   ·   {controls}")
    };
    buffer.text(
        inner.x,
        inner.y.saturating_add(inner.height.saturating_sub(1)),
        &status,
        Paint::new(
            if session.is_game_over() { GREEN } else { MUTED },
            felt,
            true,
        ),
    );
}

fn draw_holdem_card_row(
    buffer: &mut Buffer,
    area: Rect,
    cards: &[Card],
    hide: bool,
    y: u16,
    slots: usize,
) {
    let card_width = 7u16;
    let gap = 1u16;
    let total_width = card_width * slots as u16 + gap * slots.saturating_sub(1) as u16;
    let start_x = area.x + area.width.saturating_sub(total_width) / 2;
    for index in 0..slots {
        let rect = Rect::new(
            start_x + index as u16 * (card_width + gap),
            y,
            card_width,
            3,
        );
        let card = cards.get(index).copied();
        let is_hidden = hide;
        let (foreground, background, label) = if is_hidden {
            (CYAN, PANEL_ALT, "◆".to_string())
        } else if let Some(card) = card {
            let color = if matches!(card.suit, Suit::Heart | Suit::Diamond) {
                RED
            } else {
                BG
            };
            (
                color,
                Color::Rgb {
                    r: 242,
                    g: 244,
                    b: 239,
                },
                card.label(),
            )
        } else {
            (
                MUTED,
                Color::Rgb {
                    r: 10,
                    g: 50,
                    b: 42,
                },
                "·".to_string(),
            )
        };
        buffer.fill(rect, Paint::new(foreground, background, true));
        buffer.panel(
            rect,
            "",
            if is_hidden { CYAN } else { foreground },
            background,
        );
        buffer.centered_text(
            rect,
            rect.y + 1,
            &label,
            Paint::new(foreground, background, true),
        );
    }
}

fn draw_cards(buffer: &mut Buffer, cards: &[Card], x: u16, y: u16, hide_second: bool) {
    for (index, card) in cards.iter().enumerate() {
        let card_x = x.saturating_add(index as u16 * 9);
        let hidden = hide_second && index == 1;
        let background = if hidden {
            PANEL_ALT
        } else {
            Color::Rgb {
                r: 242,
                g: 244,
                b: 239,
            }
        };
        let foreground = if hidden {
            MUTED
        } else if matches!(card.suit, Suit::Heart | Suit::Diamond) {
            RED
        } else {
            BG
        };
        let rect = Rect::new(card_x, y, 7, 5);
        buffer.fill(rect, Paint::new(foreground, background, true));
        buffer.panel(
            rect,
            "",
            if hidden { BORDER } else { foreground },
            background,
        );
        let label = if hidden {
            "??".to_string()
        } else {
            card.label()
        };
        buffer.centered_text(
            rect,
            y + 2,
            &label,
            Paint::new(foreground, background, true),
        );
    }
}

fn draw_card_backs(buffer: &mut Buffer, x: u16, y: u16, count: usize) {
    for index in 0..count {
        let rect = Rect::new(x.saturating_add(index as u16 * 9), y, 7, 5);
        buffer.fill(rect, Paint::new(CYAN, PANEL_ALT, true));
        buffer.panel(rect, "", CYAN, PANEL_ALT);
        buffer.centered_text(rect, y + 2, "◆", Paint::new(CYAN, PANEL_ALT, true));
    }
}

fn draw_roulette(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.roulette_session() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.fill(
        panel,
        Paint::new(
            TEXT,
            Color::Rgb {
                r: 32,
                g: 18,
                b: 39,
            },
            false,
        ),
    );
    buffer.panel(
        panel,
        " ROULETTE // LUCKY SPIN ",
        PINK,
        Color::Rgb {
            r: 32,
            g: 18,
            b: 39,
        },
    );
    let inner = panel.inset(2);
    buffer.text(
        inner.x.saturating_add(inner.width.saturating_sub(18)),
        inner.y,
        &gambling_wager_label(app),
        Paint::new(
            YELLOW,
            Color::Rgb {
                r: 32,
                g: 18,
                b: 39,
            },
            true,
        ),
    );
    buffer.centered_text(
        inner,
        inner.y,
        if app.roulette_is_spinning() {
            "SPINNING..."
        } else {
            "PLACE YOUR BET"
        },
        Paint::new(
            TEXT,
            Color::Rgb {
                r: 32,
                g: 18,
                b: 39,
            },
            true,
        ),
    );
    let center = app.roulette_spin_frame() % WHEEL_NUMBERS.len();
    for offset in -4..=4 {
        let index = (center as isize + offset).rem_euclid(WHEEL_NUMBERS.len() as isize) as usize;
        let number = WHEEL_NUMBERS[index];
        let color = RouletteColor::for_number(number);
        let background = roulette_background(color);
        let rect = Rect::new(
            inner.x.saturating_add((offset + 4) as u16 * 7),
            inner.y + 3,
            6,
            3,
        );
        buffer.fill(rect, Paint::new(TEXT, background, true));
        buffer.centered_text(
            rect,
            rect.y + 1,
            &roulette_number_label(number),
            Paint::new(
                if offset == 0 { BG } else { TEXT },
                if offset == 0 { YELLOW } else { background },
                true,
            ),
        );
    }
    buffer.centered_text(
        inner,
        inner.y + 2,
        "▲",
        Paint::new(
            YELLOW,
            Color::Rgb {
                r: 32,
                g: 18,
                b: 39,
            },
            true,
        ),
    );
    let chip_width = 17;
    let chip_gap = 2;
    let chips_width = chip_width * 3 + chip_gap * 2;
    let chips_x = inner
        .x
        .saturating_add(inner.width.saturating_sub(chips_width) / 2);
    for (index, (color, payout)) in [
        (RouletteColor::Red, 2),
        (RouletteColor::Black, 2),
        (RouletteColor::Green, 36),
    ]
    .into_iter()
    .enumerate()
    {
        let rect = Rect::new(
            chips_x.saturating_add(index as u16 * (chip_width + chip_gap)),
            inner.y + 7,
            chip_width,
            4,
        );
        draw_roulette_chip(buffer, rect, color, session.choice() == Some(color), payout);
    }
    let choice = session
        .choice()
        .map(|color| {
            format!(
                "YOUR BET  {} {}  ·  {}  ·  WIN {}×",
                color.symbol(),
                color.label(),
                gambling_wager_label(app),
                if color == RouletteColor::Green { 36 } else { 2 }
            )
        })
        .unwrap_or_else(|| "YOUR BET  NONE  ·  TYPE AMOUNT + SELECT COLOR".to_string());
    buffer.centered_text(
        inner,
        inner.y + 11,
        &choice,
        Paint::new(
            if session.choice().is_some() {
                YELLOW
            } else {
                MUTED
            },
            Color::Rgb {
                r: 32,
                g: 18,
                b: 39,
            },
            true,
        ),
    );
    let roulette_controls = app
        .status_message()
        .unwrap_or("TYPE AMOUNT   R RED   B BLACK   G GREEN   ·   ENTER SPIN");
    buffer.text(
        inner.x,
        inner.y + 12,
        roulette_controls,
        Paint::new(
            TEXT,
            Color::Rgb {
                r: 32,
                g: 18,
                b: 39,
            },
            false,
        ),
    );
    if let Some(result_color) = session.result_color() {
        let number = session
            .result_number()
            .map(roulette_number_label)
            .unwrap_or_else(|| "?".to_string());
        buffer.text(
            inner.x,
            inner.y + 13,
            &format!(
                "RESULT  {} {}  · PAYOUT {} WON",
                number,
                result_color.label(),
                session.payout().saturating_mul(app.gambling_total_wager())
            ),
            Paint::new(
                GREEN,
                Color::Rgb {
                    r: 32,
                    g: 18,
                    b: 39,
                },
                true,
            ),
        );
    }
}

fn draw_slots(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.slots_session() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.fill(
        panel,
        Paint::new(
            TEXT,
            Color::Rgb {
                r: 46,
                g: 20,
                b: 35,
            },
            false,
        ),
    );
    buffer.panel(
        panel,
        " SLOTS // LUCKY TRIPLE ",
        PINK,
        Color::Rgb {
            r: 46,
            g: 20,
            b: 35,
        },
    );
    let inner = panel.inset(2);
    buffer.centered_text(
        inner,
        inner.y,
        "THREE REELS // ONE PULL",
        Paint::new(
            TEXT,
            Color::Rgb {
                r: 46,
                g: 20,
                b: 35,
            },
            true,
        ),
    );
    buffer.text(
        inner.x.saturating_add(inner.width.saturating_sub(16)),
        inner.y,
        &gambling_wager_label(app),
        Paint::new(
            YELLOW,
            Color::Rgb {
                r: 46,
                g: 20,
                b: 35,
            },
            true,
        ),
    );
    for (index, symbol) in session.symbols().iter().enumerate() {
        let rect = Rect::new(
            inner.x.saturating_add(5 + index as u16 * 13),
            inner.y + 3,
            11,
            7,
        );
        buffer.fill(
            rect,
            Paint::new(
                YELLOW,
                Color::Rgb {
                    r: 240,
                    g: 234,
                    b: 210,
                },
                true,
            ),
        );
        buffer.panel(
            rect,
            "",
            PINK,
            Color::Rgb {
                r: 240,
                g: 234,
                b: 210,
            },
        );
        buffer.centered_text(
            rect,
            rect.y + 2,
            symbol.label(),
            Paint::new(
                PINK,
                Color::Rgb {
                    r: 240,
                    g: 234,
                    b: 210,
                },
                true,
            ),
        );
        buffer.centered_text(
            rect,
            rect.y + 4,
            &format!("REEL {}", index + 1),
            Paint::new(
                MUTED,
                Color::Rgb {
                    r: 240,
                    g: 234,
                    b: 210,
                },
                false,
            ),
        );
    }
    let message = if session.is_game_over() {
        format!(
            "{}  ·  PAYOUT  {} WON",
            gambling_wager_label(app),
            session.payout().saturating_mul(app.gambling_total_wager())
        )
    } else if let Some(message) = app.status_message() {
        message.to_string()
    } else if app.gambling_bet_committed() {
        format!(
            "{}  ·  ENTER / SPACE  PULL LEVER",
            gambling_wager_label(app)
        )
    } else {
        "TYPE BET AMOUNT  ·  ENTER / SPACE  PULL LEVER".to_string()
    };
    buffer.centered_text(
        inner,
        inner.y + 12,
        &message,
        Paint::new(
            if session.payout() > 0 { GREEN } else { YELLOW },
            Color::Rgb {
                r: 46,
                g: 20,
                b: 35,
            },
            true,
        ),
    );
}

fn draw_typing(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.typing_session() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.panel(
        panel,
        " TYPING MINE // PRECISION RUN ",
        Color::Rgb {
            r: 255,
            g: 154,
            b: 103,
        },
        SURFACE,
    );
    let inner = panel.inset(2);
    buffer.text(
        inner.x,
        inner.y,
        "TRANSMISSION",
        Paint::new(MUTED, PANEL, true),
    );
    wrapped_text(
        buffer,
        inner,
        inner.y + 2,
        session.sentence(),
        Paint::new(TEXT, PANEL, true),
    );
    let input_rect = Rect::new(inner.x, inner.y + 6, inner.width, 3);
    buffer.panel(input_rect, " INPUT ", BORDER_BRIGHT, PANEL_ALT);
    buffer.text(
        input_rect.x + 2,
        input_rect.y + 1,
        session.input(),
        Paint::new(TEXT, PANEL_ALT, false),
    );
    let feedback = match session.last_correct() {
        Some(true) => "MATCH // +1 WON",
        Some(false) => "MISMATCH // NO REWARD",
        None => "TYPE THE TRANSMISSION EXACTLY",
    };
    buffer.text(
        inner.x,
        inner.y + 11,
        feedback,
        Paint::new(
            if session.last_correct() == Some(true) {
                GREEN
            } else {
                MUTED
            },
            PANEL,
            true,
        ),
    );
    buffer.text(
        inner.x,
        inner.y + 13,
        &format!(
            "MINED  {} WON   ATTEMPTS  {}   WALLET  {} WON",
            session.earned_won(),
            session.attempts(),
            app.wallet_won()
        ),
        Paint::new(YELLOW, PANEL, false),
    );
}

fn draw_breakout(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.breakout_session() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.panel(panel, " BREAKOUT // WALL BREAKER ", BLUE, SURFACE);
    let inner = panel.inset(2);
    let board = Rect::new(
        inner.x,
        inner.y,
        BREAKOUT_BOARD_WIDTH as u16 * 2 + 2,
        BREAKOUT_BOARD_HEIGHT as u16 + 2,
    );
    buffer.panel(board, " BOARD ", BORDER, PANEL);
    let cells = board.inset(1);
    let ball = session.ball();
    for y in 0..BREAKOUT_BOARD_HEIGHT {
        for x in 0..BREAKOUT_BOARD_WIDTH {
            let px = cells.x + x as u16 * 2;
            let py = cells.y + y as u16;
            if ball.x == x && ball.y == y {
                buffer.text(px, py, "● ", Paint::new(TEXT, RED, true));
            } else if y == session.paddle_y()
                && (session.paddle_x()..session.paddle_x() + PADDLE_WIDTH).contains(&x)
            {
                buffer.text(px, py, "██", Paint::new(BG, CYAN, true));
            } else if session.brick_at(x, y) {
                let color = match y {
                    0 => RED,
                    1 => YELLOW,
                    2 => GREEN,
                    _ => BLUE,
                };
                buffer.text(px, py, "██", Paint::new(BG, color, true));
            } else {
                buffer.text(px, py, "  ", Paint::new(GRID, PANEL, false));
            }
        }
    }
    let hud = Rect::new(
        board.x + board.width + 2,
        inner.y,
        inner.width.saturating_sub(board.width + 2),
        board.height,
    );
    draw_simple_hud(
        buffer,
        hud,
        " HUD ",
        &[
            ("SCORE", session.score().to_string(), YELLOW),
            ("LIVES", session.lives().to_string(), RED),
            ("BRICKS", session.bricks_remaining().to_string(), BLUE),
        ],
        "ARROWS / A-D MOVE PADDLE",
    );
}

fn draw_minesweeper(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.minesweeper_session() else {
        return;
    };
    let panel = centered(area, area.width.saturating_sub(2), area.height);
    buffer.panel(panel, " MINESWEEPER // SAFE FIELD ", CYAN, SURFACE);
    let inner = panel.inset(1);
    let board_width = session.width() as u16;
    let cell_width = if inner.width >= board_width.saturating_mul(2).saturating_add(20) {
        2
    } else {
        1
    };
    let board = Rect::new(
        inner.x,
        inner.y,
        board_width.saturating_mul(cell_width).saturating_add(2),
        session.height() as u16 + 2,
    );
    buffer.panel(board, " MINE GRID ", BORDER_BRIGHT, PANEL);
    for y in 0..session.height() {
        for x in 0..session.width() {
            let index = y * session.width() + x;
            let cell = session.cell(x, y).expect("cell inside board");
            let selected = index == session.cursor() && !session.is_game_over();
            let rect = Rect::new(
                board.x + 1 + x as u16 * cell_width,
                board.y + 1 + y as u16,
                cell_width,
                1,
            );
            let (glyph, foreground, background) = if session.is_game_over() && cell.is_mine {
                ("✹", BG, if selected { YELLOW } else { RED })
            } else {
                match cell.state {
                    MineCellState::Hidden => (
                        if cell_width == 2 { "▒▒" } else { "▒" },
                        if selected { BG } else { BORDER_BRIGHT },
                        if selected { CYAN } else { PANEL_ALT },
                    ),
                    MineCellState::Flagged => (
                        "⚑",
                        if selected { BG } else { YELLOW },
                        if selected { CYAN } else { PANEL_ALT },
                    ),
                    MineCellState::Revealed if cell.is_mine => ("✹", BG, RED),
                    MineCellState::Revealed if cell.adjacent_mines == 0 => {
                        ("·", GRID, if selected { PANEL_HOVER } else { SURFACE })
                    }
                    MineCellState::Revealed => (
                        mine_number(cell.adjacent_mines),
                        mine_number_color(cell.adjacent_mines),
                        if selected { PANEL_HOVER } else { SURFACE },
                    ),
                }
            };
            buffer.fill(rect, Paint::new(foreground, background, true));
            let glyph_x = rect.x + rect.width.saturating_sub(glyph.width() as u16) / 2;
            buffer.text(
                glyph_x,
                rect.y,
                glyph,
                Paint::new(foreground, background, true),
            );
        }
    }
    let hud = Rect::new(
        board.x + board.width + 1,
        inner.y,
        inner.width.saturating_sub(board.width + 1),
        board.height.min(inner.height),
    );
    let remaining = session.mine_count().saturating_sub(session.flags());
    let status = match session.outcome() {
        Some(MinesweeperOutcome::Cleared) => "FIELD CLEAR",
        Some(MinesweeperOutcome::Exploded) => "MINE HIT",
        None => "SCANNING",
    };
    draw_simple_hud(
        buffer,
        hud,
        " FIELD DATA ",
        &[
            ("MODE", session.difficulty().label().to_string(), CYAN),
            (
                "STATUS",
                status.to_string(),
                if session.is_game_over() { RED } else { GREEN },
            ),
            ("MINES", remaining.to_string(), YELLOW),
            (
                "SAFE",
                format!(
                    "{} / {}",
                    session.revealed_safe(),
                    session.width() * session.height() - session.mine_count()
                ),
                BLUE,
            ),
        ],
        "ENTER REVEAL · F FLAG",
    );
}

fn mine_number(value: u8) -> &'static str {
    match value {
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        _ => "8",
    }
}

fn mine_number_color(value: u8) -> Color {
    match value {
        1 => BLUE,
        2 => GREEN,
        3 => RED,
        4 => PURPLE,
        5 => PINK,
        _ => YELLOW,
    }
}

fn draw_connect_four(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.connect_four_session() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(1),
    );
    buffer.panel(panel, " CONNECT FOUR // NEON DROP ", YELLOW, SURFACE);
    let inner = panel.inset(2);
    let slot_width = 5;
    let board = Rect::new(
        inner.x,
        inner.y,
        CONNECT_FOUR_WIDTH as u16 * slot_width + 2,
        CONNECT_FOUR_HEIGHT as u16 * 2 + 3,
    );
    let board_background = Color::Rgb {
        r: 22,
        g: 62,
        b: 132,
    };
    buffer.panel(board, " DROP ZONE ", BLUE, board_background);
    for column in 0..CONNECT_FOUR_WIDTH {
        let x = board.x + 1 + column as u16 * slot_width;
        let selected = column == session.cursor_column() && !session.is_game_over();
        buffer.centered_text(
            Rect::new(x, board.y + 1, slot_width, 1),
            board.y + 1,
            if selected { "▼" } else { "·" },
            Paint::new(
                if selected { YELLOW } else { BORDER },
                board_background,
                true,
            ),
        );
    }
    for row in 0..CONNECT_FOUR_HEIGHT {
        for column in 0..CONNECT_FOUR_WIDTH {
            let x = board.x + 1 + column as u16 * slot_width;
            let y = board.y + 2 + row as u16 * 2;
            let slot = Rect::new(x, y, slot_width, 2);
            buffer.fill(slot, Paint::new(TEXT, board_background, false));
            let (glyph, color) = match session.board()[row * CONNECT_FOUR_WIDTH + column] {
                ConnectFourCell::Empty => ("●", Color::Rgb { r: 8, g: 15, b: 31 }),
                ConnectFourCell::Player => ("●", RED),
                ConnectFourCell::Cpu => ("●", YELLOW),
            };
            buffer.centered_text(slot, y, glyph, Paint::new(color, board_background, true));
            buffer.centered_text(slot, y + 1, "●", Paint::new(color, board_background, true));
        }
    }
    let hud = Rect::new(
        board.x + board.width + 2,
        inner.y,
        inner.width.saturating_sub(board.width + 2),
        board.height,
    );
    let status = match session.outcome() {
        Some(ConnectFourOutcome::PlayerWin) => "YOU WIN",
        Some(ConnectFourOutcome::CpuWin) => "CPU WINS",
        Some(ConnectFourOutcome::Draw) => "DRAW",
        None => "YOUR TURN",
    };
    draw_simple_hud(
        buffer,
        hud,
        " DUEL HUD ",
        &[
            ("AI", session.difficulty().label().to_string(), YELLOW),
            (
                "STATUS",
                status.to_string(),
                if session.outcome() == Some(ConnectFourOutcome::PlayerWin) {
                    GREEN
                } else {
                    CYAN
                },
            ),
            ("MOVES", session.player_moves().to_string(), BLUE),
            ("YOU / CPU", "●  /  ●".to_string(), RED),
        ],
        "← → SELECT · ENTER DROP",
    );
}

fn draw_memory_match(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.memory_match_session() else {
        return;
    };
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(1),
    );
    buffer.panel(panel, " MEMORY // SIGNAL PAIRS ", PURPLE, SURFACE);
    let inner = panel.inset(2);
    let card_width = 7;
    let card_height = 3;
    let board = Rect::new(
        inner.x,
        inner.y,
        session.width() as u16 * card_width,
        session.height() as u16 * card_height,
    );
    for y in 0..session.height() {
        for x in 0..session.width() {
            let index = y * session.width() + x;
            let card = session.cards()[index];
            let selected = index == session.cursor() && !session.awaiting_continue();
            let rect = Rect::new(
                board.x + x as u16 * card_width,
                board.y + y as u16 * card_height,
                card_width.saturating_sub(1),
                card_height,
            );
            let (background, border, glyph, foreground) = match card.state {
                MemoryCardState::Hidden => (
                    PANEL_ALT,
                    if selected { YELLOW } else { BORDER_BRIGHT },
                    "◇",
                    if selected { YELLOW } else { CYAN },
                ),
                MemoryCardState::Revealed => (
                    Color::Rgb {
                        r: 49,
                        g: 38,
                        b: 87,
                    },
                    YELLOW,
                    memory_symbol(card.symbol),
                    memory_symbol_color(card.symbol),
                ),
                MemoryCardState::Matched => (
                    Color::Rgb {
                        r: 20,
                        g: 76,
                        b: 62,
                    },
                    GREEN,
                    memory_symbol(card.symbol),
                    TEXT,
                ),
            };
            buffer.panel(rect, "", border, background);
            buffer.centered_text(
                rect,
                rect.y + 1,
                glyph,
                Paint::new(foreground, background, true),
            );
        }
    }
    let hud = Rect::new(
        board.x + board.width + 2,
        inner.y,
        inner.width.saturating_sub(board.width + 2),
        board.height.max(12).min(inner.height),
    );
    let status = if session.awaiting_continue() {
        "MEMORIZE · ENTER"
    } else {
        "FIND THE PAIR"
    };
    draw_simple_hud(
        buffer,
        hud,
        " MEMORY CORE ",
        &[
            ("MODE", session.difficulty().label().to_string(), PURPLE),
            ("STATUS", status.to_string(), CYAN),
            (
                "PAIRS",
                format!("{} / {}", session.matched_pairs(), session.pair_count()),
                GREEN,
            ),
            ("MOVES", session.moves().to_string(), YELLOW),
        ],
        "ENTER FLIP / CONTINUE",
    );
}

fn memory_symbol(symbol: u8) -> &'static str {
    const SYMBOLS: [&str; 12] = ["◆", "●", "▲", "■", "✦", "✚", "◈", "⬟", "☾", "☀", "♠", "♥"];
    SYMBOLS[usize::from(symbol) % SYMBOLS.len()]
}

fn memory_symbol_color(symbol: u8) -> Color {
    match symbol % 6 {
        0 => CYAN,
        1 => YELLOW,
        2 => RED,
        3 => GREEN,
        4 => PINK,
        _ => BLUE,
    }
}

fn draw_maze(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(session) = app.maze_session() else {
        return;
    };
    let panel = centered(area, area.width.saturating_sub(2), area.height);
    buffer.panel(panel, " MAZE // NEON LABYRINTH ", GREEN, SURFACE);
    let inner = panel.inset(1);
    let maze_width = session.width() as u16;
    let cell_width = if inner.width >= maze_width.saturating_mul(2).saturating_add(20) {
        2
    } else {
        1
    };
    let board = Rect::new(
        inner.x,
        inner.y,
        maze_width.saturating_mul(cell_width),
        session.height() as u16,
    );
    for y in 0..session.height() {
        for x in 0..session.width() {
            let index = y * session.width() + x;
            let rect = Rect::new(
                board.x + x as u16 * cell_width,
                board.y + y as u16,
                cell_width,
                1,
            );
            let (glyph, foreground, background) = if index == session.player() {
                ("◆", BG, YELLOW)
            } else if index == session.goal() {
                ("◎", BG, GREEN)
            } else if session.is_wall(index) {
                (
                    " ",
                    BLUE,
                    if (x + y) % 2 == 0 {
                        Color::Rgb {
                            r: 35,
                            g: 86,
                            b: 140,
                        }
                    } else {
                        Color::Rgb {
                            r: 28,
                            g: 69,
                            b: 119,
                        }
                    },
                )
            } else if session.was_visited(index) {
                ("·", CYAN, Color::Rgb { r: 8, g: 31, b: 39 })
            } else {
                (" ", MUTED, BG)
            };
            buffer.fill(rect, Paint::new(foreground, background, true));
            let glyph_x = rect.x + rect.width.saturating_sub(glyph.width() as u16) / 2;
            buffer.text(
                glyph_x,
                rect.y,
                glyph,
                Paint::new(foreground, background, true),
            );
        }
    }
    let hud = Rect::new(
        board.x + board.width + 2,
        inner.y,
        inner.width.saturating_sub(board.width + 2),
        board.height.min(inner.height),
    );
    let efficiency = if session.steps() == 0 {
        100
    } else {
        (u64::from(session.optimal_steps()) * 100 / u64::from(session.steps()).max(1)).min(100)
    };
    draw_simple_hud(
        buffer,
        hud,
        " PATHFINDER ",
        &[
            ("ZONE", session.difficulty().label().to_string(), GREEN),
            ("STEPS", session.steps().to_string(), YELLOW),
            ("PAR", session.optimal_steps().to_string(), BLUE),
            ("EFFICIENCY", format!("{}%", efficiency), CYAN),
        ],
        "◆ YOU   ◎ EXIT",
    );
}

fn draw_result(buffer: &mut Buffer, app: &App, area: Rect) {
    let Some(result) = app.result() else {
        return;
    };
    let kind = app.catalog().find_game(&result.game_id).and_then(|game| {
        game.stages
            .iter()
            .find(|stage| stage.id == result.stage_id)
            .map(|stage| stage.game_kind)
            .or(Some(game.kind))
    });
    let panel = centered(
        area,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    buffer.panel(
        panel,
        " GAME OVER // SCOREBOARD ",
        kind.map(game_color).unwrap_or(CYAN),
        SURFACE,
    );
    let inner = panel.inset(2);
    let left = Rect::new(inner.x, inner.y, inner.width / 2, inner.height);
    let right = Rect::new(
        inner.x + inner.width / 2 + 1,
        inner.y,
        inner.width.saturating_sub(inner.width / 2 + 1),
        inner.height,
    );
    buffer.panel(left, " FINAL SCORE ", BORDER, PANEL);
    let status_color = match result.status {
        RoundStatus::Completed => GREEN,
        RoundStatus::TimedOut => YELLOW,
        RoundStatus::Abandoned => RED,
    };
    buffer.centered_text(
        left,
        left.y + 2,
        result.status.label(),
        Paint::new(status_color, PANEL, true),
    );
    for (index, line) in result_lines(result, kind).iter().enumerate() {
        buffer.text(
            left.x + 2,
            left.y + 5 + index as u16,
            line,
            Paint::new(TEXT, PANEL, false),
        );
    }
    buffer.centered_text(
        left,
        left.y + left.height.saturating_sub(3),
        "ENTER / ESC  BACK TO ARCADE",
        Paint::new(BG, CYAN, true),
    );
    buffer.panel(right, " HIGH SCORES ", BORDER, PANEL);
    let records = app.recent_records(5);
    if records.is_empty() {
        buffer.text(
            right.x + 2,
            right.y + 3,
            "NO RUNS YET",
            Paint::new(MUTED, PANEL, false),
        );
    } else {
        for (index, record) in records.iter().enumerate() {
            let y = right.y + 2 + index as u16 * 2;
            buffer.text(
                right.x + 2,
                y,
                &format!("{:02}  {}", index + 1, record.game_id),
                Paint::new(TEXT, PANEL, true),
            );
            buffer.text(
                right.x + 2,
                y + 1,
                &format!("{} pts  ·  {}", record.score, record.status.label()),
                Paint::new(MUTED, PANEL, false),
            );
        }
    }
}

fn draw_simple_hud(
    buffer: &mut Buffer,
    rect: Rect,
    title: &str,
    rows: &[(&str, String, Color)],
    footer: &str,
) {
    buffer.panel(rect, title, BORDER, PANEL);
    let inner = rect.inset(2);
    let footer_y = inner.y + inner.height.saturating_sub(1);
    for (index, (label, value, color)) in rows.iter().enumerate() {
        let y = inner.y + index as u16 * 2;
        if y.saturating_add(1) >= footer_y {
            break;
        }
        buffer.text(inner.x, y, label, Paint::new(MUTED, PANEL, false));
        buffer.text(inner.x, y + 1, value, Paint::new(*color, PANEL, true));
    }
    buffer.text(inner.x, footer_y, footer, Paint::new(MUTED, PANEL, false));
}

fn draw_bar(buffer: &mut Buffer, rect: Rect, ratio: f64, foreground: Color, background: Color) {
    let filled = (f64::from(rect.width) * ratio.clamp(0.0, 1.0)).round() as u16;
    buffer.fill(rect, Paint::new(foreground, background, false));
    if filled > 0 {
        buffer.fill(
            Rect::new(rect.x, rect.y, filled.min(rect.width), rect.height),
            Paint::new(BG, foreground, false),
        );
    }
}

fn wrapped_text(buffer: &mut Buffer, rect: Rect, y: u16, value: &str, paint: Paint) {
    let mut line = String::new();
    let mut line_y = y;
    for word in value.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };
        if candidate.width() as u16 > rect.width && !line.is_empty() {
            buffer.text(rect.x, line_y, &line, paint);
            line_y = line_y.saturating_add(1);
            line.clear();
        }
        if line.is_empty() {
            line.push_str(word);
        } else {
            line.push(' ');
            line.push_str(word);
        }
        if line_y >= rect.y.saturating_add(rect.height) {
            break;
        }
    }
    if !line.is_empty() && line_y < rect.y.saturating_add(rect.height) {
        buffer.text(rect.x, line_y, &line, paint);
    }
}

fn centered(area: Rect, max_width: u16, max_height: u16) -> Rect {
    let width = area.width.min(max_width);
    let height = area.height.min(max_height);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

fn game_icon(kind: Option<GameKind>) -> &'static str {
    match kind {
        Some(GameKind::Arithmetic) => "∑",
        Some(GameKind::Snake) => "≈",
        Some(GameKind::TicTacToe) => "×",
        Some(GameKind::TwentyFortyEight) => "▦",
        Some(GameKind::Sudoku) => "▤",
        Some(GameKind::Gambling) => "♠",
        Some(GameKind::Blackjack) => "♣",
        Some(GameKind::Roulette) => "◉",
        Some(GameKind::Slots) => "▥",
        Some(GameKind::Holdem) => "♦",
        Some(GameKind::TypingPractice) => "⌨",
        Some(GameKind::Breakout) => "▰",
        Some(GameKind::Minesweeper) => "✹",
        Some(GameKind::ConnectFour) => "●",
        Some(GameKind::MemoryMatch) => "◆",
        Some(GameKind::Maze) => "⌗",
        None => "◇",
    }
}

fn game_color(kind: GameKind) -> Color {
    match kind {
        GameKind::Arithmetic => BLUE,
        GameKind::Snake => GREEN,
        GameKind::TicTacToe => CYAN,
        GameKind::TwentyFortyEight => YELLOW,
        GameKind::Sudoku => PURPLE,
        GameKind::Gambling
        | GameKind::Blackjack
        | GameKind::Roulette
        | GameKind::Slots
        | GameKind::Holdem => PINK,
        GameKind::TypingPractice => Color::Rgb {
            r: 255,
            g: 154,
            b: 103,
        },
        GameKind::Breakout => Color::Rgb {
            r: 112,
            g: 169,
            b: 255,
        },
        GameKind::Minesweeper => CYAN,
        GameKind::ConnectFour => YELLOW,
        GameKind::MemoryMatch => PURPLE,
        GameKind::Maze => GREEN,
    }
}

fn stage_badge(difficulty: u32) -> &'static str {
    match difficulty {
        0 => "--",
        1 => "01",
        2 => "02",
        3 => "03",
        _ => "++",
    }
}

fn tile_colors(tile: u32) -> (Color, Color) {
    match tile {
        0 => (MUTED, GRID),
        2 => (
            BG,
            Color::Rgb {
                r: 215,
                g: 225,
                b: 235,
            },
        ),
        4 => (
            BG,
            Color::Rgb {
                r: 190,
                g: 207,
                b: 226,
            },
        ),
        8 => (
            BG,
            Color::Rgb {
                r: 255,
                g: 182,
                b: 92,
            },
        ),
        16 => (
            BG,
            Color::Rgb {
                r: 255,
                g: 143,
                b: 91,
            },
        ),
        32 => (
            BG,
            Color::Rgb {
                r: 255,
                g: 104,
                b: 105,
            },
        ),
        64 => (
            TEXT,
            Color::Rgb {
                r: 225,
                g: 75,
                b: 107,
            },
        ),
        128 => (
            BG,
            Color::Rgb {
                r: 185,
                g: 143,
                b: 255,
            },
        ),
        256 => (
            TEXT,
            Color::Rgb {
                r: 145,
                g: 104,
                b: 234,
            },
        ),
        512 => (
            BG,
            Color::Rgb {
                r: 88,
                g: 215,
                b: 180,
            },
        ),
        1024 => (
            BG,
            Color::Rgb {
                r: 71,
                g: 188,
                b: 204,
            },
        ),
        _ => (
            BG,
            Color::Rgb {
                r: 255,
                g: 211,
                b: 73,
            },
        ),
    }
}

fn tile_border(tile: u32, shimmer: bool) -> Color {
    if tile == 0 {
        return if shimmer { BORDER_BRIGHT } else { GRID };
    }
    match tile {
        2 | 4 => BLUE,
        8 | 16 => YELLOW,
        32 | 64 => RED,
        128 | 256 => PURPLE,
        512 | 1024 => CYAN,
        _ => GREEN,
    }
}

fn tile_highlight(tile: u32, shimmer: bool) -> Color {
    if tile == 0 {
        return if shimmer { BORDER_BRIGHT } else { BORDER };
    }
    match tile {
        2 => Color::Rgb {
            r: 245,
            g: 250,
            b: 255,
        },
        4 => Color::Rgb {
            r: 224,
            g: 238,
            b: 255,
        },
        8 | 16 => Color::Rgb {
            r: 255,
            g: 218,
            b: 132,
        },
        32 | 64 => Color::Rgb {
            r: 255,
            g: 153,
            b: 157,
        },
        128 | 256 => Color::Rgb {
            r: 220,
            g: 194,
            b: 255,
        },
        512 | 1024 => Color::Rgb {
            r: 171,
            g: 255,
            b: 231,
        },
        _ => Color::Rgb {
            r: 255,
            g: 239,
            b: 143,
        },
    }
}

fn tile_shadow(tile: u32) -> Color {
    match tile {
        0 => BG,
        2 => Color::Rgb {
            r: 151,
            g: 164,
            b: 181,
        },
        4 => Color::Rgb {
            r: 124,
            g: 146,
            b: 174,
        },
        8 | 16 => Color::Rgb {
            r: 191,
            g: 101,
            b: 48,
        },
        32 | 64 => Color::Rgb {
            r: 176,
            g: 48,
            b: 78,
        },
        128 | 256 => Color::Rgb {
            r: 102,
            g: 65,
            b: 166,
        },
        512 | 1024 => Color::Rgb {
            r: 36,
            g: 131,
            b: 125,
        },
        _ => Color::Rgb {
            r: 183,
            g: 137,
            b: 30,
        },
    }
}

fn direction_label(direction: Direction) -> &'static str {
    match direction {
        Direction::Up => "UP",
        Direction::Down => "DOWN",
        Direction::Left => "LEFT",
        Direction::Right => "RIGHT",
    }
}

fn roulette_background(color: RouletteColor) -> Color {
    match color {
        RouletteColor::Red => Color::Rgb {
            r: 151,
            g: 34,
            b: 54,
        },
        RouletteColor::Black => Color::Rgb {
            r: 25,
            g: 30,
            b: 39,
        },
        RouletteColor::Green => Color::Rgb {
            r: 18,
            g: 119,
            b: 78,
        },
    }
}

fn draw_roulette_chip(
    buffer: &mut Buffer,
    rect: Rect,
    color: RouletteColor,
    selected: bool,
    payout: u16,
) {
    let base = roulette_background(color);
    let background = if selected { YELLOW } else { base };
    let foreground = if selected { BG } else { TEXT };
    buffer.fill(rect, Paint::new(foreground, background, true));
    buffer.panel(rect, "", if selected { YELLOW } else { BORDER }, background);
    buffer.centered_text(
        rect,
        rect.y.saturating_add(1),
        &format!("{} {}", color.symbol(), color.label()),
        Paint::new(foreground, background, true),
    );
    buffer.centered_text(
        rect,
        rect.y.saturating_add(2),
        &format!("{}× PAYOUT", payout),
        Paint::new(foreground, background, false),
    );
}

fn roulette_number_label(number: u8) -> String {
    if number == 37 {
        "00".to_string()
    } else {
        number.to_string()
    }
}

fn gambling_feedback_line(app: &App) -> Option<String> {
    app.gambling_feedback().map(str::to_string)
}

fn gambling_wager_label(app: &App) -> String {
    if app.gambling_bet_committed() {
        format!("BET {} WON", app.gambling_total_wager())
    } else if app.gambling_bet_input().is_empty() {
        "BET [____] WON".to_string()
    } else {
        format!("BET [{}] WON", app.gambling_bet_input())
    }
}

fn result_lines(result: &crate::round::RoundResult, kind: Option<GameKind>) -> Vec<String> {
    let time = format!(
        "PLAY TIME  {:.1}s",
        result.actual_play_time_ms as f64 / 1000.0
    );
    match kind {
        Some(GameKind::Snake) => vec![
            format!("FOOD  {}", result.score),
            format!("SCORE  {}", result.score),
            time,
        ],
        Some(GameKind::TicTacToe) => vec![
            format!(
                "RESULT  {}",
                if result.score > 0 { "WIN" } else { "GAME OVER" }
            ),
            format!("SCORE  {}", result.score),
            time,
        ],
        Some(GameKind::TwentyFortyEight) => vec![
            format!("SCORE  {}", result.score),
            format!("MOVES  {}", result.attempts),
            time,
        ],
        Some(GameKind::Sudoku) => vec![
            format!("SCORE  {}", result.score),
            format!("CORRECT  {}", result.correct_answers),
            format!("ACCURACY  {}%", result.accuracy_percent()),
            time,
        ],
        Some(GameKind::Blackjack) => vec![
            format!(
                "RESULT  {}",
                if result.score == 2 {
                    "BLACKJACK"
                } else if result.score == 1 {
                    "WIN"
                } else {
                    "DEFEAT"
                }
            ),
            format!("ACTIONS  {}", result.attempts),
            time,
        ],
        Some(GameKind::Roulette) | Some(GameKind::Slots) => vec![
            format!("PAYOUT  {} WON", result.score),
            format!("ACCURACY  {}%", result.accuracy_percent()),
            time,
        ],
        Some(GameKind::Holdem) => vec![
            format!("PAYOUT  {} WON", result.score),
            format!("ACTIONS  {}", result.attempts),
            time,
        ],
        Some(GameKind::TypingPractice) => vec![
            format!("MINED  {} WON", result.correct_answers),
            format!("ACCURACY  {}%", result.accuracy_percent()),
            time,
        ],
        Some(GameKind::Breakout) => vec![
            format!("SCORE  {}", result.score),
            format!("BRICKS  {}", result.correct_answers),
            time,
        ],
        Some(GameKind::Minesweeper) => vec![
            format!("SCORE  {}", result.score),
            format!("SAFE ACTIONS  {}", result.correct_answers),
            format!("ACCURACY  {}%", result.accuracy_percent()),
            time,
        ],
        Some(GameKind::ConnectFour) => vec![
            format!("SCORE  {}", result.score),
            format!("MOVES  {}", result.attempts),
            time,
        ],
        Some(GameKind::MemoryMatch) => vec![
            format!("SCORE  {}", result.score),
            format!("PAIRS  {}", result.correct_answers),
            format!("MOVES  {}", result.attempts),
            time,
        ],
        Some(GameKind::Maze) => vec![
            format!("SCORE  {}", result.score),
            format!("STEPS  {}", result.attempts),
            format!("EFFICIENCY  {}%", result.accuracy_percent()),
            time,
        ],
        _ => vec![
            format!("CORRECT  {}", result.correct_answers),
            format!("ACCURACY  {}%", result.accuracy_percent()),
            format!("STREAK  {}", result.best_streak),
            format!("SCORE  {}", result.score),
            time,
        ],
    }
}

fn game_summary(game: &GameDefinition) -> &'static str {
    match game.kind {
        GameKind::Arithmetic => "시간 안에 최대한 많은 문제를 풀어 점수를 올립니다.",
        GameKind::Snake => "먹이를 먹으며 몸을 키우고 오래 생존합니다.",
        GameKind::TicTacToe => "컴퓨터와 번갈아 두며 3칸을 먼저 연결합니다.",
        GameKind::TwentyFortyEight => "타일을 합쳐 더 큰 숫자를 만들고 2048에 도전합니다.",
        GameKind::Sudoku => "빈칸을 채워 모든 가로·세로·3×3 박스를 완성합니다.",
        GameKind::Gambling => "블랙잭·룰렛·슬롯·홀덤을 즐기는 아케이드 카지노입니다.",
        GameKind::Blackjack => "카드를 받아 21에 가까워지고 딜러를 이깁니다.",
        GameKind::Roulette => "색을 선택하고 룰렛 결과가 맞으면 배당금을 받습니다.",
        GameKind::Slots => "세 칸의 그림을 맞춰 배당금을 받습니다.",
        GameKind::Holdem => "AI와 프리플랍부터 리버까지 겨루고 쇼다운에서 족보를 비교합니다.",
        GameKind::TypingPractice => "문장을 정확히 입력할 때마다 1원을 채굴합니다.",
        GameKind::Breakout => "패들로 공을 튕겨 모든 벽돌을 파괴합니다.",
        GameKind::Minesweeper => "숫자 단서를 읽고 깃발을 세워 안전 구역을 모두 확보합니다.",
        GameKind::ConnectFour => "AI보다 먼저 네 개의 디스크를 가로·세로·대각선으로 연결합니다.",
        GameKind::MemoryMatch => "카드의 위치를 기억해 모든 심볼 쌍을 최소 이동으로 찾습니다.",
        GameKind::Maze => "자동 생성된 네온 미로에서 흔적을 따라 출구까지 탈출합니다.",
    }
}

fn stage_description(kind: GameKind, stage: &StageDefinition) -> &'static str {
    match kind {
        GameKind::Arithmetic => operation_description(stage.operation),
        GameKind::Snake => "벽과 자신의 몸을 피하며 먹이를 먹는 클래식 보드입니다.",
        GameKind::TicTacToe => "3×3 보드에서 X와 O가 한 줄을 완성하기 위해 대결합니다.",
        GameKind::TwentyFortyEight => "4×4 보드에서 같은 숫자의 타일을 합칩니다.",
        GameKind::Sudoku => "9×9 보드의 모든 행·열·3×3 박스에 1부터 9를 한 번씩 넣습니다.",
        GameKind::Gambling => "도박장 메뉴에서 게임을 선택합니다.",
        GameKind::Blackjack => "히트로 카드를 받고 스탠드로 딜러와 결과를 비교합니다.",
        GameKind::Roulette => "빨강·검정은 2배, 초록은 36배입니다.",
        GameKind::Slots => "같은 그림이 많을수록 높은 배당을 받습니다.",
        GameKind::Holdem => "C/Enter 체크·콜 · R 레이즈 · F 폴드 · AI 난이도는 매 판 랜덤입니다.",
        GameKind::TypingPractice => "문장 전체가 정확히 일치해야 1원을 받습니다.",
        GameKind::Breakout => "자동으로 움직이는 공을 패들로 받아 벽돌을 모두 부숩니다.",
        GameKind::Minesweeper => {
            "첫 선택은 항상 안전하며, 단계마다 보드 크기와 지뢰 수가 증가합니다."
        }
        GameKind::ConnectFour => "단계에 따라 랜덤 AI부터 5수 앞을 읽는 AI까지 대결합니다.",
        GameKind::MemoryMatch => "틀린 두 카드는 Enter를 눌러 확인한 뒤 다시 덮습니다.",
        GameKind::Maze => "매 판 새로 생성되는 완전 미로에서 최단 경로에 도전합니다.",
    }
}

fn game_controls(kind: GameKind) -> &'static str {
    match kind {
        GameKind::Arithmetic => "숫자 입력 · Backspace 수정 · Enter 제출",
        GameKind::Snake => "방향키/WASD 이동 · Esc 중단",
        GameKind::TicTacToe => "방향키 칸 선택 · Enter 놓기 · Esc 중단",
        GameKind::TwentyFortyEight => "방향키/WASD 이동 · Esc 중단",
        GameKind::Sudoku => "방향키 칸 선택 · 숫자 입력 · Backspace 지우기",
        GameKind::Gambling => "스테이지를 선택해 도박 게임을 시작합니다.",
        GameKind::Blackjack => "테이블에서 숫자 베팅 · Enter/H 히트 · S/Space 스탠드",
        GameKind::Roulette => "테이블에서 숫자 베팅 · R/B/G 색 선택 · Enter 돌리기",
        GameKind::Slots => "테이블에서 숫자 베팅 · Enter/Space 레버 당기기",
        GameKind::Holdem => "테이블에서 숫자 베팅 · C 체크 · R/T/Y 추가 베팅 · F 폴드",
        GameKind::TypingPractice => "문장 입력 · Enter 제출 · Backspace 수정",
        GameKind::Breakout => "방향키/A-D 패들 이동 · Esc 중단",
        GameKind::Minesweeper => "방향키/WASD 이동 · Enter 공개 · F 깃발",
        GameKind::ConnectFour => "좌우 칸 선택 · Enter 디스크 놓기",
        GameKind::MemoryMatch => "방향키/WASD 카드 선택 · Enter 뒤집기",
        GameKind::Maze => "방향키/WASD 이동 · Esc 중단",
    }
}

fn operation_description(operation: crate::arithmetic::Operation) -> &'static str {
    match operation {
        crate::arithmetic::Operation::Addition => "0~9 + 0~9",
        crate::arithmetic::Operation::Subtraction => "0~9 − 0~9 (음수 없음)",
        crate::arithmetic::Operation::Multiplication => "0~9 × 0~9",
        crate::arithmetic::Operation::Division => "정확히 나누어지는 정수 문제",
    }
}

fn stage_time_limit(stage: &StageDefinition) -> String {
    if stage.time_limit.is_zero() {
        "무제한".to_string()
    } else {
        format!("{}초", stage.time_limit.as_secs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cabinet_window_keeps_late_games_visible() {
        assert_eq!(cabinet_window_start(0, 11, 7), 0);
        assert_eq!(cabinet_window_start(5, 11, 7), 2);
        assert_eq!(cabinet_window_start(10, 11, 7), 4);
        assert_eq!(cabinet_window_start(0, 0, 0), 0);
    }
}
