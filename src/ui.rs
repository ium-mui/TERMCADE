use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::{App, Screen};
use crate::blackjack::Card;
use crate::breakout::{BOARD_HEIGHT as BREAKOUT_BOARD_HEIGHT, BOARD_WIDTH as BREAKOUT_BOARD_WIDTH};
use crate::domain::{GameDefinition, GameKind, StageDefinition};
use crate::game_2048::GRID_SIZE;
use crate::roulette::WHEEL_NUMBERS;
use crate::round::RoundStatus;
use crate::snake::{BOARD_HEIGHT, BOARD_WIDTH, Point};
use crate::sudoku::SUDOKU_SIZE;
use crate::tictactoe::Cell as TicTacToeCell;

const BG: Color = Color::Rgb(12, 16, 24);
const PANEL: Color = Color::Rgb(20, 27, 39);
const PANEL_ALT: Color = Color::Rgb(25, 34, 49);
const BORDER: Color = Color::Rgb(61, 79, 106);
const ACCENT: Color = Color::Rgb(82, 190, 255);
const MUTED: Color = Color::Rgb(139, 155, 178);

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::default().bg(BG)), area);

    if area.width < 44 || area.height < 20 {
        draw_small_terminal(frame, area);
        return;
    }

    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .areas(area);

    draw_header(frame, app, header);
    match app.screen {
        Screen::GameSelect => draw_game_select(frame, app, body),
        Screen::StageSelect => draw_stage_select(frame, app, body),
        Screen::Ready => draw_ready(frame, app, body),
        Screen::Playing => draw_playing(frame, app, body),
        Screen::Result => draw_result(frame, app, body),
    }
    draw_footer(frame, app, footer);
}

fn draw_small_terminal(frame: &mut Frame<'_>, area: Rect) {
    let text = vec![
        Line::from(Span::styled(
            "터미널 창이 너무 작습니다",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("최소 44×20 크기로 늘려 주세요."),
        Line::from("게임 보드가 잘리지 않도록 창을 키운 뒤 계속할 수 있습니다."),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" CLI GAME ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(BORDER))
                    .style(Style::default().bg(PANEL)),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_header(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let screen_title = match app.screen {
        Screen::GameSelect => "게임 선택",
        Screen::StageSelect => "스테이지 선택",
        Screen::Ready => "준비",
        Screen::Playing => "플레이",
        Screen::Result => "결과",
    };
    let game_title = app
        .selected_game_definition()
        .map(|game| game.display_name.as_str())
        .unwrap_or("아케이드");
    let line = Line::from(vec![
        Span::styled(
            " CLI GAME ",
            Style::default()
                .fg(BG)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(screen_title, Style::default().fg(Color::White)),
        Span::styled("  /  ", Style::default().fg(MUTED)),
        Span::styled(game_title, Style::default().fg(ACCENT)),
        Span::raw("    "),
        Span::styled(
            format!("지갑 {}원", app.wallet_won()),
            Style::default().fg(Color::Yellow),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(line).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(BORDER)),
        ),
        area,
    );
}

fn draw_game_select(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let panel = centered_panel(area, 96, 22);
    let inner = screen_inner(frame, panel);
    let [list_area, _, detail_area] = if inner.width >= 72 {
        Layout::horizontal([
            Constraint::Length(34),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .areas(inner)
    } else {
        Layout::horizontal([
            Constraint::Percentage(100),
            Constraint::Length(0),
            Constraint::Length(0),
        ])
        .areas(inner)
    };

    let items = app
        .catalog
        .games()
        .iter()
        .map(|game| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:<12}", game.display_name),
                    Style::default().fg(Color::White),
                ),
                Span::styled(game.id.to_string(), Style::default().fg(MUTED)),
            ]))
        })
        .collect::<Vec<_>>();
    let list = List::new(items)
        .block(
            Block::default()
                .title(" 게임 ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER)),
        )
        .highlight_style(
            Style::default()
                .fg(BG)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    let mut state = ListState::default();
    state.select((!app.catalog.games().is_empty()).then_some(app.selected_game));
    frame.render_stateful_widget(list, list_area, &mut state);

    if detail_area.width > 0 {
        let Some(game) = app.selected_game_definition() else {
            return;
        };
        let text = vec![
            Line::from(Span::styled(
                game.display_name.as_str(),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("ID  {}", game.id)),
            Line::from(""),
            Line::from(game_summary(game)),
            Line::from(""),
            Line::from(Span::styled(
                "Enter  스테이지 선택",
                Style::default().fg(Color::Green),
            )),
        ];
        frame.render_widget(
            Paragraph::new(text)
                .block(
                    Block::default()
                        .title(" 게임 정보 ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(BORDER)),
                )
                .wrap(Wrap { trim: true }),
            detail_area,
        );
    }
}

fn draw_stage_select(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(game) = app.selected_game_definition() else {
        return;
    };
    let panel = centered_panel(area, 96, 22);
    let inner = screen_inner(frame, panel);
    let [list_area, _, detail_area] = if inner.width >= 72 {
        Layout::horizontal([
            Constraint::Length(42),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .areas(inner)
    } else {
        Layout::horizontal([
            Constraint::Percentage(100),
            Constraint::Length(0),
            Constraint::Length(0),
        ])
        .areas(inner)
    };

    let items = game
        .stages
        .iter()
        .map(|stage| {
            ListItem::new(vec![
                Line::from(Span::styled(
                    format!("  {}", stage.display_name),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    format!("     {}  ·  {}", stage.id, stage_time_limit(stage)),
                    Style::default().fg(MUTED),
                )),
            ])
        })
        .collect::<Vec<_>>();
    let list = List::new(items)
        .block(
            Block::default()
                .title(" 스테이지 ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER)),
        )
        .highlight_style(
            Style::default()
                .fg(BG)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    let mut state = ListState::default();
    state.select((!game.stages.is_empty()).then_some(app.selected_stage));
    frame.render_stateful_widget(list, list_area, &mut state);

    if detail_area.width > 0 {
        let Some(stage) = app.selected_stage_definition() else {
            return;
        };
        let text = vec![
            Line::from(Span::styled(
                stage.display_name.as_str(),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("ID       {}", stage.id)),
            Line::from(format!("난이도   {}단계", stage.difficulty_order)),
            Line::from(format!("시간     {}", stage_time_limit(stage))),
            Line::from(""),
            Line::from(stage_description(stage.game_kind, stage)),
            Line::from(""),
            Line::from(Span::styled(
                "Enter  준비 화면",
                Style::default().fg(Color::Green),
            )),
        ];
        frame.render_widget(
            Paragraph::new(text)
                .block(
                    Block::default()
                        .title(" 스테이지 정보 ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(BORDER)),
                )
                .wrap(Wrap { trim: true }),
            detail_area,
        );
    }
}

fn draw_ready(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(game) = app.selected_game_definition() else {
        return;
    };
    let Some(stage) = app.selected_stage_definition() else {
        return;
    };
    let panel = centered_panel(area, 76, 19);
    let mut lines = vec![
        Line::from(Span::styled(
            stage.display_name.as_str(),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(stage_description(stage.game_kind, stage)),
        Line::from(""),
        Line::from(game_controls(stage.game_kind)),
        Line::from(""),
        best_record_line(app, &game.id, &stage.id),
        Line::from(""),
        Line::from(Span::styled(
            "Enter를 누르면 시작합니다.",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    if let Some(message) = app.status_message.as_deref() {
        lines.push(Line::from(Span::styled(
            message,
            Style::default().fg(Color::Red),
        )));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(Line::from(format!(" {} · READY ", game.display_name)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(ACCENT)),
            )
            .wrap(Wrap { trim: true }),
        panel,
    );
}

fn draw_playing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    if app.snake_session.is_some() {
        draw_snake_playing(frame, app, area);
    } else if app.tictactoe_session.is_some() {
        draw_tictactoe_playing(frame, app, area);
    } else if app.game2048_session.is_some() {
        draw_2048_playing(frame, app, area);
    } else if app.sudoku_session.is_some() {
        draw_sudoku_playing(frame, app, area);
    } else if app.blackjack_session.is_some() {
        draw_blackjack_playing(frame, app, area);
    } else if app.roulette_session.is_some() {
        draw_roulette_playing(frame, app, area);
    } else if app.slots_session.is_some() {
        draw_slots_playing(frame, app, area);
    } else if app.typing_session.is_some() {
        draw_typing_playing(frame, app, area);
    } else if app.breakout_session.is_some() {
        draw_breakout_playing(frame, app, area);
    } else {
        draw_math_playing(frame, app, area);
    }
}

fn draw_math_playing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(session) = app.session.as_ref() else {
        return;
    };
    let panel = centered_panel(area, 100, 22);
    let inner = screen_inner(frame, panel);
    let [question_area, _, stats_area] = if inner.width >= 82 {
        Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(30),
        ])
        .areas(inner)
    } else {
        Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(0),
            Constraint::Length(7),
        ])
        .areas(inner)
    };

    let question_inner = panel_inner(frame, question_area, Line::from(" 문제 "));
    let feedback = match session.last_answer_correct() {
        Some(true) => Span::styled("정답입니다!", Style::default().fg(Color::Green)),
        Some(false) => Span::styled("오답입니다. 다음 문제", Style::default().fg(Color::Yellow)),
        None => Span::styled("문제를 풀어 주세요", Style::default().fg(MUTED)),
    };
    let answer = if session.answer_input().is_empty() {
        "_"
    } else {
        session.answer_input()
    };
    let question_lines = vec![
        Line::from(Span::styled(
            session.current_problem().equation(),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("답안  ", Style::default().fg(MUTED)),
            Span::styled(
                format!("  {answer}  "),
                Style::default()
                    .fg(Color::White)
                    .bg(PANEL_ALT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(feedback),
        app.status_message.as_deref().map_or_else(
            || Line::from(""),
            |message| Line::from(Span::styled(message, Style::default().fg(Color::Green))),
        ),
    ];
    frame.render_widget(
        Paragraph::new(question_lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        question_inner,
    );
    draw_math_stats(frame, session, stats_area);
}

fn draw_math_stats(frame: &mut Frame<'_>, session: &crate::round::RoundSession, area: Rect) {
    let inner = panel_inner(frame, area, Line::from(" 기록 "));
    let time_limit = session.time_limit().as_secs_f64().max(1.0);
    let ratio = (session.remaining().as_secs_f64() / time_limit).clamp(0.0, 1.0);
    let stats = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(inner);
    frame.render_widget(
        Gauge::default()
            .block(Block::default().title(" 남은 시간 "))
            .gauge_style(Style::default().fg(if ratio < 0.25 { Color::Red } else { ACCENT }))
            .label(format!("{}초", session.remaining_seconds()))
            .ratio(ratio),
        stats[0],
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(format!("정답       {}", session.correct_answers())),
            Line::from(format!("시도       {}", session.attempts())),
            Line::from(format!("현재 연속  {}", session.current_streak())),
            Line::from(format!("최고 연속  {}", session.best_streak())),
        ])
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true }),
        stats[2],
    );
}

fn draw_snake_playing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(session) = app.snake_session.as_ref() else {
        return;
    };
    let panel = centered_panel(area, 104, 21);
    let inner = screen_inner(frame, panel);
    let [board_area, _, info_area] = if inner.width >= 58 {
        Layout::horizontal([
            Constraint::Length(56),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(inner)
    } else {
        Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(0),
            Constraint::Length(0),
        ])
        .areas(inner)
    };
    draw_snake_board(frame, session, board_area);
    if info_area.height > 0 {
        draw_game_info(
            frame,
            info_area,
            vec![
                Line::from(Span::styled(
                    format!("점수  {}", session.score()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("방향키 또는 W/A/S/D"),
                Line::from("벽·몸에 닿으면 종료"),
                Line::from("Esc  게임 중단"),
            ],
        );
    }
}

fn draw_snake_board(frame: &mut Frame<'_>, session: &crate::snake::SnakeSession, area: Rect) {
    let inner = panel_inner(
        frame,
        area,
        Line::from(format!(" BOARD · 점수 {} ", session.score())),
    );
    let compact = inner.width < BOARD_WIDTH as u16 * 2;
    let head = session.snake().first().copied();
    let food = session.food();
    let mut lines = Vec::with_capacity(BOARD_HEIGHT as usize);
    for y in 0..BOARD_HEIGHT {
        let mut spans = Vec::with_capacity(BOARD_WIDTH as usize);
        for x in 0..BOARD_WIDTH {
            let point = Point { x, y };
            let (symbol, style) = if head == Some(point) {
                (
                    "@",
                    Style::default()
                        .fg(BG)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else if session.snake().contains(&point) {
                ("o", Style::default().fg(Color::Green))
            } else if food == point {
                (
                    "*",
                    Style::default()
                        .fg(BG)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ("·", Style::default().fg(Color::Rgb(49, 65, 87)))
            };
            if compact {
                spans.push(Span::styled(symbol, style));
            } else {
                spans.push(Span::styled(format!("{symbol} "), style));
            }
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}

fn draw_tictactoe_playing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(session) = app.tictactoe_session.as_ref() else {
        return;
    };
    let panel = centered_panel(area, 88, 19);
    let inner = screen_inner(frame, panel);
    let [board_area, _, info_area] = if inner.width >= 60 {
        Layout::horizontal([
            Constraint::Length(31),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(inner)
    } else {
        Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(0),
            Constraint::Length(0),
        ])
        .areas(inner)
    };
    draw_tictactoe_board(frame, session, board_area);
    if info_area.height > 0 {
        draw_game_info(
            frame,
            info_area,
            vec![
                Line::from(Span::styled(
                    "당신의 말  X",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled(
                    "컴퓨터의 말  O",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from("방향키 선택 · Enter 놓기"),
                Line::from("Esc  게임 중단"),
            ],
        );
    }
}

fn draw_tictactoe_board(
    frame: &mut Frame<'_>,
    session: &crate::tictactoe::TicTacToeSession,
    area: Rect,
) {
    let inner = panel_inner(frame, area, Line::from(" BOARD "));
    let mut lines = Vec::with_capacity(5);
    lines.push(Line::from("+-----+-----+-----+"));
    for y in 0..3 {
        let mut spans = vec![Span::raw("|")];
        for x in 0..3 {
            let index = y * 3 + x;
            let (symbol, style) = match session.board()[index] {
                TicTacToeCell::Empty => ("·", Style::default().fg(MUTED)),
                TicTacToeCell::X => (
                    "X",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                TicTacToeCell::O => (
                    "O",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            };
            let style = if session.cursor() == index {
                style.bg(Color::Rgb(45, 64, 89))
            } else {
                style
            };
            spans.push(Span::styled(format!("  {symbol}  "), style));
            spans.push(Span::raw("|"));
        }
        lines.push(Line::from(spans));
        lines.push(Line::from("+-----+-----+-----+"));
    }
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}

fn draw_2048_playing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(session) = app.game2048_session.as_ref() else {
        return;
    };
    let panel = centered_panel(area, 92, 21);
    let inner = screen_inner(frame, panel);
    let [board_area, _, info_area] = if inner.width >= 64 {
        Layout::horizontal([
            Constraint::Length(39),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(inner)
    } else {
        Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(0),
            Constraint::Length(0),
        ])
        .areas(inner)
    };
    draw_2048_board(frame, session, board_area);
    if info_area.height > 0 {
        draw_game_info(
            frame,
            info_area,
            vec![
                Line::from(Span::styled(
                    format!("점수  {}", session.score()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(format!("이동  {}", session.moves())),
                Line::from("방향키 또는 W/A/S/D"),
                Line::from("2048을 만들면 승리"),
                Line::from("Esc  게임 중단"),
            ],
        );
    }
}

fn draw_2048_board(frame: &mut Frame<'_>, session: &crate::game_2048::Game2048Session, area: Rect) {
    let inner = panel_inner(
        frame,
        area,
        Line::from(format!(" BOARD · {}점 ", session.score())),
    );
    let separator = "+-------+-------+-------+-------+";
    let mut lines = vec![Line::from(separator)];
    for row in session.board() {
        let mut spans = vec![Span::raw("|")];
        for tile in row.iter().take(GRID_SIZE) {
            let (text, style) = if *tile == 0 {
                (
                    "·".to_string(),
                    Style::default().fg(Color::Rgb(94, 111, 136)),
                )
            } else {
                (tile.to_string(), tile_style(*tile))
            };
            spans.push(Span::styled(format!(" {text:^5} "), style));
            spans.push(Span::raw("|"));
        }
        lines.push(Line::from(spans));
        lines.push(Line::from(separator));
    }
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}

fn draw_sudoku_playing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(session) = app.sudoku_session.as_ref() else {
        return;
    };
    let panel = centered_panel(area, 96, 21);
    let inner = screen_inner(frame, panel);
    let [board_area, _, info_area] = if inner.width >= 72 {
        Layout::horizontal([
            Constraint::Length(41),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(inner)
    } else {
        Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(0),
            Constraint::Length(0),
        ])
        .areas(inner)
    };
    draw_sudoku_board(frame, session, board_area);
    if info_area.height > 0 {
        draw_game_info(
            frame,
            info_area,
            vec![
                Line::from(Span::styled(
                    format!(
                        "완성  {}/{}",
                        session.filled_count(),
                        SUDOKU_SIZE * SUDOKU_SIZE
                    ),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(format!("오류  {}", session.mistakes())),
                Line::from(format!("시도  {}", session.attempts())),
                Line::from("방향키 이동 · 숫자 입력"),
                Line::from("Backspace 지우기 · Esc 중단"),
            ],
        );
    }
}

fn draw_sudoku_board(frame: &mut Frame<'_>, session: &crate::sudoku::SudokuSession, area: Rect) {
    let inner = panel_inner(
        frame,
        area,
        Line::from(format!(" BOARD · 오류 {} ", session.mistakes())),
    );
    let mut lines = vec![sudoku_separator()];
    for row in 0..SUDOKU_SIZE {
        let mut spans = vec![Span::raw("|")];
        for column in 0..SUDOKU_SIZE {
            let index = row * SUDOKU_SIZE + column;
            let value = session.board()[index];
            let symbol = if value == 0 {
                "·".to_string()
            } else {
                value.to_string()
            };
            let mut style = if session.is_given(index) {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if session.is_wrong(index) || session.is_conflict(index) {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else if value != 0 {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(MUTED)
            };
            if index == session.cursor() {
                style = style.bg(Color::Rgb(45, 64, 89));
            }
            spans.push(Span::styled(format!(" {symbol} "), style));
            spans.push(Span::raw("|"));
        }
        lines.push(Line::from(spans));
        if row == 2 || row == 5 || row == 8 {
            lines.push(sudoku_separator());
        }
    }
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}

fn sudoku_separator() -> Line<'static> {
    let mut text = String::from("+");
    for column in 0..SUDOKU_SIZE {
        text.push_str("---");
        if column == SUDOKU_SIZE - 1 || (column + 1) % 3 == 0 {
            text.push('+');
        } else {
            text.push('-');
        }
    }
    Line::from(text)
}

fn draw_blackjack_playing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(session) = app.blackjack_session.as_ref() else {
        return;
    };
    let panel = centered_panel(area, 100, 21);
    let inner = screen_inner(frame, panel);
    let [table_area, _, info_area] = if inner.width >= 66 {
        Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(30),
        ])
        .areas(inner)
    } else {
        Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(0),
            Constraint::Length(0),
        ])
        .areas(inner)
    };

    let table_inner = panel_inner(frame, table_area, Line::from(" TABLE "));
    let dealer_score = if session.dealer_revealed() {
        session.dealer_score().to_string()
    } else {
        session.dealer_visible_score().to_string()
    };
    let dealer_cards = card_line(session.dealer_cards(), !session.dealer_revealed());
    let player_cards = card_line(session.player_cards(), false);
    let outcome = session
        .outcome()
        .map(|outcome| Span::styled(outcome.label(), Style::default().fg(Color::Green)))
        .unwrap_or_else(|| Span::styled("진행 중", Style::default().fg(MUTED)));
    let mut lines = vec![
        Line::from(Span::styled(
            "DEALER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        dealer_cards,
        Line::from(format!("공개 점수  {dealer_score}")),
        Line::from(""),
        Line::from(Span::styled(
            "PLAYER",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        player_cards,
        Line::from(format!("점수       {}", session.player_score())),
        Line::from(""),
        Line::from(vec![
            Span::styled("상태       ", Style::default().fg(MUTED)),
            outcome,
        ]),
    ];
    if let Some(line) = gambling_feedback_line(app) {
        lines.push(line);
    }
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        table_inner,
    );

    if info_area.height > 0 {
        draw_game_info(
            frame,
            info_area,
            vec![
                Line::from(Span::styled(
                    "21에 가까운 쪽 승리",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(format!("행동  {}", session.attempts())),
                Line::from("Enter/H  히트"),
                Line::from("S/Space  스탠드"),
                Line::from("Esc  게임 중단"),
            ],
        );
    }
}

fn draw_roulette_playing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(session) = app.roulette_session.as_ref() else {
        return;
    };
    let panel = centered_panel(area, 100, 19);
    let inner = screen_inner(frame, panel);
    let [table_area, _, info_area] = if inner.width >= 64 {
        Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(30),
        ])
        .areas(inner)
    } else {
        Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(0),
            Constraint::Length(0),
        ])
        .areas(inner)
    };
    let title = if app.roulette_is_spinning() {
        " ROULETTE · SPINNING "
    } else {
        " ROULETTE "
    };
    let table_inner = panel_inner(frame, table_area, Line::from(title));
    let choice = session
        .choice()
        .map(|color| format!("{} {}", color.symbol(), color.label()))
        .unwrap_or_else(|| "선택 안 함".to_string());
    let mut lines = vec![
        Line::from(Span::styled(
            if app.roulette_is_spinning() {
                "룰렛이 회전 중입니다..."
            } else {
                "어느 색에 1원을 걸까요?"
            },
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        roulette_wheel_line(app),
        Line::from(Span::styled(
            "            ▲",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[1] 빨강  ", Style::default().fg(Color::Red)),
            Span::styled("[2] 검정  ", Style::default().fg(Color::White)),
            Span::styled("[3] 초록", Style::default().fg(Color::Green)),
        ]),
        Line::from(format!("현재 선택  {choice}")),
        Line::from(""),
        Line::from(Span::styled(
            "Enter / Space  룰렛 돌리기",
            Style::default().fg(Color::Green),
        )),
    ];
    if let Some(result_color) = session.result_color() {
        let result_number = session
            .result_number()
            .map(roulette_number_label)
            .unwrap_or_else(|| "?".to_string());
        lines.push(Line::from(format!(
            "결과  {} {} · 배당 {}원",
            result_number,
            result_color.label(),
            session.payout()
        )));
    }
    if let Some(line) = gambling_feedback_line(app) {
        lines.push(line);
    }
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        table_inner,
    );
    if info_area.height > 0 {
        draw_game_info(
            frame,
            info_area,
            vec![
                Line::from(Span::styled(
                    "배당 안내",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from("빨강/검정  2배"),
                Line::from("초록        36배"),
                Line::from(format!("보유        {}원", app.wallet_won())),
                Line::from("Esc  게임 중단"),
            ],
        );
    }
}

fn roulette_wheel_line(app: &App) -> Line<'static> {
    let center = app.roulette_spin_frame() % WHEEL_NUMBERS.len();
    let mut spans = Vec::new();
    for offset in -4..=4 {
        let index = (center as isize + offset).rem_euclid(WHEEL_NUMBERS.len() as isize) as usize;
        let number = WHEEL_NUMBERS[index];
        let color = crate::roulette::RouletteColor::for_number(number);
        let mut style = match color {
            crate::roulette::RouletteColor::Red => Style::default().fg(Color::Red),
            crate::roulette::RouletteColor::Black => Style::default().fg(Color::White),
            crate::roulette::RouletteColor::Green => Style::default().fg(Color::Green),
        };
        if offset == 0 {
            style = style
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD);
        }
        spans.push(Span::styled(
            format!(" {:^2} ", roulette_number_label(number)),
            style,
        ));
    }
    Line::from(spans)
}

fn roulette_number_label(number: u8) -> String {
    if number == 37 {
        "00".to_string()
    } else {
        number.to_string()
    }
}

fn draw_slots_playing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(session) = app.slots_session.as_ref() else {
        return;
    };
    let panel = centered_panel(area, 100, 19);
    let inner = screen_inner(frame, panel);
    let [machine_area, _, info_area] = if inner.width >= 64 {
        Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(30),
        ])
        .areas(inner)
    } else {
        Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(0),
            Constraint::Length(0),
        ])
        .areas(inner)
    };
    let machine_inner = panel_inner(frame, machine_area, Line::from(" SLOTS "));
    let symbols = session.symbols();
    let symbol_spans: Vec<Span<'static>> = symbols
        .iter()
        .flat_map(|symbol| {
            [
                Span::styled(
                    format!("  {}  ", symbol.label()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("│", Style::default().fg(BORDER)),
            ]
        })
        .collect();
    let mut lines = vec![
        Line::from(Span::styled(
            "행운의 세 칸",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(symbol_spans),
        Line::from(""),
        Line::from(Span::styled(
            "Enter / Space  레버 당기기",
            Style::default().fg(Color::Green),
        )),
    ];
    if session.is_game_over() {
        lines.push(Line::from(format!("결과  배당 {}원", session.payout())));
    }
    if let Some(line) = gambling_feedback_line(app) {
        lines.push(line);
    }
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        machine_inner,
    );
    if info_area.height > 0 {
        draw_game_info(
            frame,
            info_area,
            vec![
                Line::from(Span::styled(
                    "배당 안내",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from("7 7 7       20배"),
                Line::from("같은 그림  8배"),
                Line::from("두 그림     2배"),
                Line::from(format!("보유        {}원", app.wallet_won())),
                Line::from("Esc  게임 중단"),
            ],
        );
    }
}

fn draw_typing_playing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(session) = app.typing_session.as_ref() else {
        return;
    };
    let panel = centered_panel(area, 100, 20);
    let inner = screen_inner(frame, panel);
    let [practice_area, _, info_area] = if inner.width >= 68 {
        Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(30),
        ])
        .areas(inner)
    } else {
        Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(0),
            Constraint::Length(0),
        ])
        .areas(inner)
    };
    let practice_inner = panel_inner(frame, practice_area, Line::from(" TYPING MINE "));
    let input = if session.input().is_empty() {
        "_"
    } else {
        session.input()
    };
    let feedback = match session.last_correct() {
        Some(true) => Span::styled("정확합니다! 1원 획득", Style::default().fg(Color::Green)),
        Some(false) => Span::styled("틀렸습니다. 보상 없음", Style::default().fg(Color::Yellow)),
        None => Span::styled("문장을 그대로 입력하세요", Style::default().fg(MUTED)),
    };
    let lines = vec![
        Line::from(Span::styled(
            "다음 문장",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(session.sentence()),
        Line::from(""),
        Line::from(vec![
            Span::styled("입력  ", Style::default().fg(MUTED)),
            Span::styled(
                input.to_string(),
                Style::default().fg(Color::White).bg(PANEL_ALT),
            ),
        ]),
        Line::from(""),
        Line::from(feedback),
        Line::from("Enter 제출 · Backspace 수정"),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        practice_inner,
    );
    if info_area.height > 0 {
        draw_game_info(
            frame,
            info_area,
            vec![
                Line::from(Span::styled(
                    format!("채굴  {}원", session.earned_won()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(format!("보유  {}원", app.wallet_won())),
                Line::from(format!("시도  {}", session.attempts())),
                Line::from("정확히 일치할 때만 지급"),
                Line::from("Esc  채굴 종료"),
            ],
        );
    }
}

fn card_line(cards: &[Card], hide_second: bool) -> Line<'static> {
    let spans: Vec<Span<'static>> = cards
        .iter()
        .enumerate()
        .map(|(index, card)| {
            let label = if hide_second && index == 1 {
                "[??]".to_string()
            } else {
                format!("[{}]", card.label())
            };
            let color = if matches!(
                card.suit,
                crate::blackjack::Suit::Heart | crate::blackjack::Suit::Diamond
            ) {
                Color::Red
            } else {
                Color::White
            };
            Span::styled(
                format!("{label} "),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )
        })
        .collect();
    Line::from(spans)
}

fn gambling_feedback_line(app: &App) -> Option<Line<'static>> {
    app.gambling_feedback.as_ref().map(|feedback| {
        Line::from(Span::styled(
            feedback.clone(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))
    })
}

fn draw_breakout_playing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(session) = app.breakout_session.as_ref() else {
        return;
    };
    let panel = centered_panel(area, 100, 21);
    let inner = screen_inner(frame, panel);
    let [board_area, _, info_area] = if inner.width >= 64 {
        Layout::horizontal([
            Constraint::Length(40),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(inner)
    } else {
        Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(0),
            Constraint::Length(0),
        ])
        .areas(inner)
    };
    draw_breakout_board(frame, session, board_area);
    if info_area.height > 0 {
        draw_game_info(
            frame,
            info_area,
            vec![
                Line::from(Span::styled(
                    format!("점수  {}", session.score()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(format!("목숨  {}", session.lives())),
                Line::from(format!("벽돌  {}", session.bricks_remaining())),
                Line::from("←/A   왼쪽 이동"),
                Line::from("→/D   오른쪽 이동"),
                Line::from("Esc  게임 중단"),
            ],
        );
    }
}

fn draw_breakout_board(
    frame: &mut Frame<'_>,
    session: &crate::breakout::BreakoutSession,
    area: Rect,
) {
    let inner = panel_inner(
        frame,
        area,
        Line::from(format!(" BOARD · {}점 ", session.score())),
    );
    let ball = session.ball();
    let mut lines = Vec::with_capacity(BREAKOUT_BOARD_HEIGHT as usize);
    for y in 0..BREAKOUT_BOARD_HEIGHT {
        let mut spans = Vec::with_capacity(BREAKOUT_BOARD_WIDTH as usize);
        for x in 0..BREAKOUT_BOARD_WIDTH {
            let (symbol, style) = if ball.x == x && ball.y == y {
                (
                    "o ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )
            } else if y == session.paddle_y()
                && (session.paddle_x()..session.paddle_x() + crate::breakout::PADDLE_WIDTH)
                    .contains(&x)
            {
                (
                    "==",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else if session.brick_at(x, y) {
                let color = match y {
                    0 => Color::Red,
                    1 => Color::Yellow,
                    2 => Color::Green,
                    _ => Color::Blue,
                };
                (
                    "[]",
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )
            } else {
                ("  ", Style::default().fg(Color::Rgb(49, 65, 87)))
            };
            spans.push(Span::styled(symbol, style));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}

fn draw_result(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(result) = app.result.as_ref() else {
        return;
    };
    let kind = app.catalog.find_game(&result.game_id).map(|game| {
        game.stages
            .iter()
            .find(|stage| stage.id == result.stage_id)
            .map(|stage| stage.game_kind)
            .unwrap_or(game.kind)
    });
    let panel = centered_panel(area, 100, 22);
    let inner = screen_inner(frame, panel);
    let [summary_area, _, recent_area] = if inner.width >= 78 {
        Layout::horizontal([
            Constraint::Length(42),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(inner)
    } else {
        Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(0),
            Constraint::Length(7),
        ])
        .areas(inner)
    };

    let status_style = match result.status {
        RoundStatus::Completed => Style::default().fg(Color::Green),
        RoundStatus::TimedOut => Style::default().fg(Color::Yellow),
        RoundStatus::Abandoned => Style::default().fg(Color::Red),
    };
    let mut lines = vec![Line::from(Span::styled(
        result.status.label(),
        status_style.add_modifier(Modifier::BOLD),
    ))];
    lines.extend(result_lines(result, kind));
    if let Some(message) = app.status_message.as_deref() {
        lines.push(Line::from(Span::styled(
            message,
            Style::default().fg(Color::Red),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter/Esc  스테이지 목록으로",
        Style::default().fg(Color::Green),
    )));
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }),
        summary_area,
    );

    let recent = app.recent_records(4);
    let recent_items = if recent.is_empty() {
        vec![ListItem::new(Line::from("아직 기록이 없습니다."))]
    } else {
        recent
            .into_iter()
            .map(|record| {
                ListItem::new(vec![
                    Line::from(Span::styled(
                        format!("{}  {}점", record.game_id, record.score),
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::styled(
                        format!(
                            "{} · {}% · {}",
                            record.stage_id,
                            record.accuracy_percent(),
                            record.status.label()
                        ),
                        Style::default().fg(MUTED),
                    )),
                ])
            })
            .collect()
    };
    frame.render_widget(
        List::new(recent_items).block(
            Block::default()
                .title(" 최근 기록 ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER)),
        ),
        recent_area,
    );
}

fn result_lines(result: &crate::round::RoundResult, kind: Option<GameKind>) -> Vec<Line<'static>> {
    let time = format!(
        "플레이 시간  {:.1}초",
        result.actual_play_time_ms as f64 / 1000.0
    );
    match kind {
        Some(GameKind::Snake) => vec![
            Line::from(format!("먹은 먹이    {}", result.score)),
            Line::from(format!("점수         {}", result.score)),
            Line::from(time),
        ],
        Some(GameKind::TicTacToe) => vec![
            Line::from(format!(
                "결과         {}",
                if result.score > 0 {
                    "승리"
                } else {
                    "게임 종료"
                }
            )),
            Line::from(format!("점수         {}", result.score)),
            Line::from(time),
        ],
        Some(GameKind::TwentyFortyEight) => vec![
            Line::from(format!("최종 점수    {}", result.score)),
            Line::from(format!("이동 횟수    {}", result.attempts)),
            Line::from(time),
        ],
        Some(GameKind::Sudoku) => vec![
            Line::from(format!("점수         {}", result.score)),
            Line::from(format!("정답 입력    {}", result.correct_answers)),
            Line::from(format!("전체 시도    {}", result.attempts)),
            Line::from(format!("정확도       {}%", result.accuracy_percent())),
            Line::from(time),
        ],
        Some(GameKind::Blackjack) => vec![
            Line::from(format!(
                "결과         {}",
                if result.status == RoundStatus::Abandoned {
                    "중단"
                } else if result.score == 2 {
                    "블랙잭"
                } else if result.score == 1 {
                    "승리"
                } else {
                    "패배 또는 무승부"
                }
            )),
            Line::from(format!("행동 횟수    {}", result.attempts)),
            Line::from(format!("점수         {}", result.score)),
            Line::from(time),
        ],
        Some(GameKind::Roulette) => vec![
            Line::from(format!("룰렛 결과    {}원", result.score)),
            Line::from(format!("정확도       {}%", result.accuracy_percent())),
            Line::from(time),
        ],
        Some(GameKind::Slots) => vec![
            Line::from(format!("슬롯 결과    {}원", result.score)),
            Line::from(format!("정확도       {}%", result.accuracy_percent())),
            Line::from(time),
        ],
        Some(GameKind::TypingPractice) => vec![
            Line::from(format!("채굴한 원    {}원", result.correct_answers)),
            Line::from(format!("전체 시도    {}", result.attempts)),
            Line::from(format!("정확도       {}%", result.accuracy_percent())),
            Line::from(time),
        ],
        Some(GameKind::Breakout) => vec![
            Line::from(format!("최종 점수    {}", result.score)),
            Line::from(format!("파괴한 벽돌  {}", result.correct_answers)),
            Line::from(time),
        ],
        _ => vec![
            Line::from(format!("정답 수      {}", result.correct_answers)),
            Line::from(format!("전체 시도    {}", result.attempts)),
            Line::from(format!("정확도       {}%", result.accuracy_percent())),
            Line::from(format!("최고 연속    {}", result.best_streak)),
            Line::from(format!("점수         {}", result.score)),
            Line::from(time),
        ],
    }
}

fn draw_game_info(frame: &mut Frame<'_>, area: Rect, lines: Vec<Line<'static>>) {
    let inner = panel_inner(frame, area, Line::from(" INFO "));
    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_footer(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let help = match app.screen {
        Screen::GameSelect | Screen::StageSelect => "↑↓ 이동   Enter 선택   Esc 이전   Ctrl+C 종료",
        Screen::Ready => "Enter 시작   Esc 이전   Ctrl+C 종료",
        Screen::Playing if app.snake_session.is_some() => {
            "방향키/WASD 이동   Esc 중단   Ctrl+C 종료"
        }
        Screen::Playing if app.tictactoe_session.is_some() => {
            "방향키 선택   Enter 놓기   Esc 중단   Ctrl+C 종료"
        }
        Screen::Playing if app.game2048_session.is_some() => {
            "방향키/WASD 이동   Esc 중단   Ctrl+C 종료"
        }
        Screen::Playing if app.sudoku_session.is_some() => {
            "방향키 이동   숫자 입력   Backspace 지우기   Esc 중단"
        }
        Screen::Playing if app.blackjack_session.is_some() => {
            "Enter/H 히트   S/Space 스탠드   Esc 도박장 나가기   Ctrl+C 종료"
        }
        Screen::Playing if app.roulette_session.is_some() => {
            "1 빨강   2 검정   3 초록   Enter 돌리기   Esc 도박장 나가기"
        }
        Screen::Playing if app.slots_session.is_some() => {
            "Enter/Space 레버 당기기   Esc 도박장 나가기   Ctrl+C 종료"
        }
        Screen::Playing if app.typing_session.is_some() => {
            "문장 입력   Enter 제출   Backspace 수정   Esc 종료"
        }
        Screen::Playing if app.breakout_session.is_some() => {
            "←→/A-D 패들 이동   Esc 중단   Ctrl+C 종료"
        }
        Screen::Playing => "숫자 입력   Backspace 수정   Enter 제출   Esc 중단",
        Screen::Result => "Enter/Esc 목록으로   q 종료",
    };
    frame.render_widget(
        Paragraph::new(help)
            .style(Style::default().fg(MUTED))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(BORDER)),
            ),
        area,
    );
}

fn panel_inner(frame: &mut Frame<'_>, area: Rect, title: Line<'static>) -> Rect {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL));
    let inner = inset(area, 1, 1);
    frame.render_widget(block, area);
    inner
}

fn screen_inner(frame: &mut Frame<'_>, area: Rect) -> Rect {
    frame.render_widget(Block::default().style(Style::default().bg(PANEL)), area);
    inset(area, 1, 1)
}

fn centered_panel(area: Rect, max_width: u16, max_height: u16) -> Rect {
    let width = area.width.min(max_width);
    let height = area.height.min(max_height);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

fn inset(area: Rect, horizontal: u16, vertical: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(horizontal),
        y: area.y.saturating_add(vertical),
        width: area.width.saturating_sub(horizontal.saturating_mul(2)),
        height: area.height.saturating_sub(vertical.saturating_mul(2)),
    }
}

fn game_summary(game: &GameDefinition) -> &'static str {
    match game.kind {
        GameKind::Arithmetic => "시간 안에 최대한 많은 문제를 풀어 점수를 올립니다.",
        GameKind::Snake => "먹이를 먹으며 몸을 키우고 오래 생존합니다.",
        GameKind::TicTacToe => "컴퓨터와 번갈아 두며 3칸을 먼저 연결합니다.",
        GameKind::TwentyFortyEight => "타일을 합쳐 더 큰 숫자를 만들고 2048에 도전합니다.",
        GameKind::Sudoku => "빈칸을 채워 모든 가로·세로·3×3 박스를 완성합니다.",
        GameKind::Gambling => "공용 원으로 블랙잭·룰렛·슬롯머신을 즐기는 도박장입니다.",
        GameKind::Blackjack => "카드를 받아 21에 가까워지고 딜러를 이깁니다.",
        GameKind::Roulette => "색을 선택하고 룰렛 결과가 맞으면 배당금을 받습니다.",
        GameKind::Slots => "세 칸의 그림을 맞춰 배당금을 받습니다.",
        GameKind::TypingPractice => "문장을 정확히 입력할 때마다 1원을 채굴합니다.",
        GameKind::Breakout => "패들로 공을 튕겨 모든 벽돌을 파괴합니다.",
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
        GameKind::TypingPractice => "문장 전체가 정확히 일치해야 1원을 받습니다.",
        GameKind::Breakout => "자동으로 움직이는 공을 패들로 받아 벽돌을 모두 부숩니다.",
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
        GameKind::Blackjack => "Enter/H 히트 · S/Space 스탠드 · Esc 중단",
        GameKind::Roulette => "1/2/3 색 선택 · Enter 돌리기 · Esc 중단",
        GameKind::Slots => "Enter/Space 레버 당기기 · Esc 중단",
        GameKind::TypingPractice => "문장 입력 · Enter 제출 · Backspace 수정",
        GameKind::Breakout => "방향키/A-D 패들 이동 · Esc 중단",
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

fn best_record_line(
    app: &App,
    game_id: &crate::domain::GameId,
    stage_id: &crate::domain::StageId,
) -> Line<'static> {
    if let Some(record) = app.best_record(game_id, stage_id) {
        Line::from(format!(
            "최고 기록  {}점 · 정확도 {}% · 연속 {}",
            record.score,
            record.accuracy_percent(),
            record.best_streak
        ))
    } else if app.history_error().is_some() {
        Line::from(Span::styled(
            "기록을 읽을 수 없습니다.",
            Style::default().fg(Color::Red),
        ))
    } else {
        Line::from("최고 기록  아직 없음")
    }
}

fn tile_style(tile: u32) -> Style {
    let color = match tile {
        2 | 4 => Color::Rgb(214, 224, 238),
        8 | 16 => Color::Rgb(255, 180, 92),
        32 | 64 => Color::Rgb(255, 108, 108),
        128 | 256 => Color::Rgb(159, 126, 255),
        512 | 1024 => Color::Rgb(79, 206, 171),
        _ => Color::Rgb(255, 213, 79),
    };
    Style::default()
        .fg(BG)
        .bg(color)
        .add_modifier(Modifier::BOLD)
}
