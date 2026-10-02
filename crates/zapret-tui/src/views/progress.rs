//! The screen a running sweep owns: one bar, one log, and a cancel key.
//!
//! The bar is a plain [`Gauge`]. What makes it work is that nothing here waits
//! on the job: the sweep runs on its own thread and pushes events into this
//! view through a mutex, while the frame loop keeps repainting on its own
//! timer. A bar that only repaints when the sweep has something to say freezes
//! through every slow step, and a frozen bar reads as a hung program.

use std::collections::VecDeque;
use std::time::Instant;

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Gauge, Paragraph, Wrap};
use ratatui::Frame;
use tui_realm_stdlib::components::Spinner;
use tuirealm::component::Component;
use unicode_width::UnicodeWidthStr;
use zapret_wrapper::autotune::{LogLevel, SweepEvent};
use zapret_wrapper::domains::TtlEvent;

use crate::theme::Theme;

use super::{clock, fit, frame};

/// How many log lines the screen keeps. Older ones fall off the top, so a long
/// sweep cannot grow without bound.
const LOG_CAPACITY: usize = 500;

/// Braille spinner: it animates from the clock rather than from a tick counter,
/// so it keeps turning even while the sweep is busy and reporting nothing.
const SPINNER: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

const SPINNER_FRAME_MS: u128 = 120;

/// How wide a long log line is allowed to get before it is wrapped.
///
/// A winws command line is a few hundred characters, and a log pane that clips
/// it shows the least interesting part.
const WRAP: usize = 72;

/// Break a long line on spaces, so nothing is cut off mid-argument.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if line.is_empty() {
            line.push_str(word);
        } else if line.width() + 1 + word.width() <= width {
            line.push(' ');
            line.push_str(word);
        } else {
            out.push(std::mem::take(&mut line));
            line.push_str(word);
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

/// What the bar's label counts.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BarFormat {
    /// A percentage, for a sweep whose steps are not a fixed small set.
    Percent,
    /// `done of total`, for a sweep over a known range such as the TTL hops.
    Count,
}

pub struct ProgressView {
    started: Instant,
    title: String,
    done: usize,
    total: usize,
    phase: String,
    format: BarFormat,
    log: VecDeque<Line<'static>>,
}

impl Default for ProgressView {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressView {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            title: rust_i18n::t!("atv_title").to_string(),
            done: 0,
            total: 0,
            phase: rust_i18n::t!("atv_starting").into_owned(),
            format: BarFormat::Percent,
            log: VecDeque::new(),
        }
    }

    pub fn titled(title: String, format: BarFormat) -> Self {
        // The title goes on the bar's top border next to a spinner, so a value
        // written for an old block title must not bring its trailing space.
        Self {
            title: title.trim().to_string(),
            format,
            ..Self::new()
        }
    }

    /// Fold one autotune event into the picture.
    pub fn apply(&mut self, event: SweepEvent) {
        match event {
            SweepEvent::Phase(name) => {
                // The bar's title only shows the step that is running, so
                // without this the log would sit empty for whole minutes.
                self.phase = name.clone();
                self.push(Line::from(Span::styled(
        format!(" ▸ {}", name),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                )));
            }
            SweepEvent::Step { done, total } => {
                self.done = done;
                // The sweep only learns its own size once it has planned, so a
                // later non-zero total replaces the placeholder.
                if total > 0 {
                    self.total = total;
                }
            }
            SweepEvent::Log { level, text } => self.push(probe_line(level, &text)),
        }
    }

    /// Fold one TTL-sweep event into the picture.
    ///
    /// The TTL sweep walks a fixed small range, so its bar counts hops rather
    /// than percentages: "7 / 20" is more useful than "35%" when the thing being
    /// counted is the answer.
    pub fn apply_ttl(&mut self, event: TtlEvent) {
        // The counter is how many hops are behind us, not the hop value: the
        // range starts above zero, so a bar reading "3 / 18" on the first hop
        // would be a hop count pretending to be progress.
        let hops = || (zapret_wrapper::domains::ttl::TTL_MAX - zapret_wrapper::domains::ttl::TTL_MIN + 1) as usize;
        let finished = |ttl: u8| (ttl as usize).saturating_sub(zapret_wrapper::domains::ttl::TTL_MIN as usize) + 1;

        match event {
            TtlEvent::Trying(ttl) => {
                self.total = hops();
                self.done = finished(ttl) - 1;
                self.phase = rust_i18n::t!("ttl_trying").replace("{}", &ttl.to_string());
                self.push(Line::from(Span::styled(
        format!(" ▸ {}", self.phase),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                )));
            }
            TtlEvent::Refused(ttl, said) => {
                self.done = finished(ttl).min(hops());
                let said = said.trim();
                if said.is_empty() {
                    self.push(probe_line(
                        LogLevel::Bad,
                        &rust_i18n::t!("ttl_log_died").replace("{}", &ttl.to_string()),
                    ));
                } else {
                    // winws prints one line and exits when it dislikes an
                    // argument. That line is the whole explanation.
                    for line in wrap(said, WRAP) {
                        self.push(Line::from(vec![
            Span::styled(" ✖ ", Theme::bad()),
                            Span::styled(line.to_string(), Style::default().fg(Color::Red)),
                        ]));
                    }
                }
            }
            TtlEvent::Probed { domain, ok } => {
                // The mark comes from the level, not the text, so a probe line
                // reads the same as every other line in the log.
                self.push(probe_line(if ok { LogLevel::Good } else { LogLevel::Bad }, &domain));
            }
            TtlEvent::Found(ttl) => {
                self.total = hops();
                self.done = finished(ttl).min(hops());
                self.phase = rust_i18n::t!("ttl_trying").replace("{}", &ttl.to_string());
                self.push(probe_line(
                    LogLevel::Good,
                    &rust_i18n::t!("ttl_log_found").replace("{}", &ttl.to_string()),
                ));
            }
        }
    }

    /// Append a line, dropping the oldest once the log is full.
    fn push(&mut self, line: Line<'static>) {
        if self.log.len() >= LOG_CAPACITY {
            self.log.pop_front();
        }
        self.log.push_back(line);
    }

    /// What the log holds, oldest first.
    ///
    /// The frame loop never needs this — it paints the view — but a test on the
    /// other side of the thread boundary has no other way to see what a worker
    /// pushed in.
    #[cfg(test)]
    pub(crate) fn log_snapshot(&self) -> Vec<String> {
        self.log
            .iter()
            .map(|line| line.spans.iter().map(|s| s.content.as_ref()).collect::<String>())
            .collect()
    }

    fn ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.done as f64 / self.total as f64).clamp(0.0, 1.0)
        }
    }

    /// `n of total` for a counted sweep, a percentage for the rest.
    fn label(&self) -> String {
        match self.format {
            BarFormat::Count => format!("{} / {}", self.done, self.total),
            BarFormat::Percent => format!("{}%", (self.ratio() * 100.0).round() as u16),
        }
    }

    /// The spinner, stepped by hand rather than per frame: this screen repaints
    /// every 50 ms and the spinner has to keep its own clock or it turns three
    /// times too fast.
    fn spinner(&self) -> Spinner {
        let mut spinner = Spinner::default()
            .sequence(SPINNER)
            .style(Theme::accent())
            .manual_step();
        let tick = (self.started.elapsed().as_millis() / SPINNER_FRAME_MS) as usize;
        for _ in 0..=tick {
            spinner.states.step();
        }
        spinner
    }

    /// Seconds left, extrapolated from the average step so far.
    fn eta(&self) -> Option<u64> {
        if self.done == 0 || self.total == 0 || self.done >= self.total {
            return None;
        }
        let per_step = self.started.elapsed().as_secs_f64() / self.done as f64;
        Some((per_step * (self.total - self.done) as f64) as u64)
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(3)
            .constraints(
                [
                    Constraint::Min(4),
                    Constraint::Length(1),
                    Constraint::Length(4),
                    Constraint::Length(1),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(area);

        // The log takes the space above and the bar sits at the bottom, next to
        // the key that stops it: the two things the user acts on stay together
        // and the eye comes back to the same place every step.
        self.render_log(f, chunks[0]);
        self.render_bar(f, chunks[2]);
        self.render_stats(f, chunks[3]);

        let help = Paragraph::new(Span::styled(
            rust_i18n::t!("atv_cancel_hint"),
            Style::default().fg(Color::Gray),
        ))
        .alignment(Alignment::Center)
        .block(frame(None));
        f.render_widget(help, chunks[4]);
    }

    fn render_bar(&self, f: &mut Frame, area: Rect) {
        // The title sits on the top border, so it has to leave the corners, the
        // spinner and the closing rule alone even when the step name is long.
        let title = format!("  {}  {}", self.title, self.phase);
        let title = fit(&title, area.width.saturating_sub(6));
        let gauge = Gauge::default()
            .block(frame(Some(Span::styled(format!("{title} "), Theme::accent()))))
            .gauge_style(Style::default().fg(Color::Cyan))
            .use_unicode(true)
            .ratio(self.ratio())
            .label(self.label());
        f.render_widget(gauge, area);
        // The spinner is its own widget in the gap the title left for it, rather
        // than a character inside the title — so it cannot be cut off by a long
        // step name and cannot make the bar's border move.
        self.spinner().view(
            f,
            Rect {
                x: area.x + 2,
                y: area.y,
                width: 1,
                height: 1,
            },
        );
    }

    fn render_stats(&self, f: &mut Frame, area: Rect) {
        let elapsed = self.started.elapsed().as_secs();
        let eta = match self.eta() {
            Some(secs) => clock(secs),
            None => rust_i18n::t!("atv_eta_unknown").into_owned(),
        };

        // A counted sweep already shows `done / total` on the bar, so repeating
        // it here would just be the same number twice.
        let mut spans = Vec::new();
        if self.format == BarFormat::Percent {
            spans.push(Span::styled(format!("{} / {}", self.done, self.total), Theme::accent()));
        spans.push(Span::raw("  ·  "));
        }
        spans.push(Span::styled(
            format!("{} {}", rust_i18n::t!("atv_elapsed"), clock(elapsed)),
            Style::default().fg(Color::White),
        ));
        spans.push(Span::raw("  ·  "));
        spans.push(Span::styled(
            format!("{} {}", rust_i18n::t!("atv_eta"), eta),
            Style::default().fg(Color::Gray),
        ));

        f.render_widget(Paragraph::new(Line::from(spans)).alignment(Alignment::Center), area);
    }

    fn render_log(&self, f: &mut Frame, area: Rect) {
        let block = frame(Some(Span::styled(rust_i18n::t!("atv_log"), Theme::title())));
        let inner = block.inner(area);
        // The log always shows its newest line: the interesting end is the one
        // the sweep is still writing to.
        let scroll = (self.log.len() as u16).saturating_sub(inner.height);
        let body = if self.log.is_empty() {
            vec![Line::from(Span::styled(
                rust_i18n::t!("atv_log_empty"),
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            self.log.iter().cloned().collect()
        };
        // Whatever still does not fit the pane is wrapped rather than cut: a log
        // line is the explanation, and half an explanation is worse than none.
        f.render_widget(
            Paragraph::new(body)
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((scroll, 0)),
            area,
        );
    }
}

/// One probe result, marked so the log can be skimmed rather than read.
fn probe_line(level: LogLevel, text: &str) -> Line<'static> {
    let (style, mark) = match level {
        LogLevel::Info => (Style::default().fg(Color::Gray), " "),
        LogLevel::Good => (Theme::ok(), "✔"),
        LogLevel::Bad => (Theme::bad(), "✖"),
    };
    Line::from(vec![
        Span::styled(format!(" {} ", mark), style),
        Span::styled(format!(" {}", text), style),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    /// A long daemon message is wrapped, not cut: the whole point of showing it
    /// is that it is the explanation.
    #[test]
    fn a_long_message_is_wrapped_not_cut() {
        let mut view = ProgressView::new();
        let said = (0..40)
            .map(|i| format!("--reason-number-{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        view.apply_ttl(TtlEvent::Refused(3, said));

        let log = view.log_snapshot();
        assert!(log.len() > 1, "a long message was not wrapped");
        for line in &log {
            assert!(line.width() <= WRAP + 4, "a wrapped line is still too wide: {line:?}");
        }
        for i in 0..40 {
            let arg = format!("--reason-number-{i}");
            assert!(log.iter().any(|l| l.contains(&arg)), "{arg} was lost");
        }
    }

    #[test]
    fn a_phase_change_also_reaches_the_log() {
        // The bar's title only ever shows the running step, so a log that did
        // not get the phases too would sit empty for whole minutes.
        let mut view = ProgressView::new();
        assert!(view.log.is_empty());

        view.apply(SweepEvent::Phase("Baseline checks — Discord".to_string()));
        assert_eq!(view.phase, "Baseline checks — Discord");
        assert_eq!(view.log.len(), 1);
        assert!(view.log[0].spans[0].content.contains("Baseline checks"));
    }

    #[test]
    fn the_log_never_grows_past_its_capacity() {
        let mut view = ProgressView::new();
        for i in 0..(LOG_CAPACITY + 50) {
            view.apply(SweepEvent::Log {
                level: LogLevel::Info,
                text: format!("line {i}"),
            });
        }
        assert_eq!(view.log.len(), LOG_CAPACITY);
        assert!(view.log[0].spans.iter().any(|s| s.content.contains("line 50")));
    }

    /// The TTL bar counts hops, not percent, and moves one hop at a time.
    #[test]
    fn the_ttl_bar_counts_hops() {
        let min = zapret_wrapper::domains::ttl::TTL_MIN;
        let hops = (zapret_wrapper::domains::ttl::TTL_MAX - min + 1) as usize;

        let mut view = ProgressView::titled("TTL".to_string(), BarFormat::Count);
        assert_eq!(view.label(), "0 / 0");

        view.apply_ttl(TtlEvent::Trying(min));
        assert_eq!(view.total, hops);
        assert_eq!(view.done, 0);
        assert_eq!(view.label(), format!("0 / {hops}"));

        view.apply_ttl(TtlEvent::Probed {
            domain: "discord.com".into(),
            ok: false,
        });
        view.apply_ttl(TtlEvent::Trying(min + 1));
        assert_eq!(view.done, 1);
        assert_eq!(view.label(), format!("1 / {hops}"));

        // A hop that found nothing still counts as time spent, so the bar moves
        // on rather than sitting on the last value.
        view.apply_ttl(TtlEvent::Refused(min + 1, String::new()));
        assert_eq!(view.done, 2);
        assert_eq!(view.label(), format!("2 / {hops}"));

        // The sweep stops at the first hop that works, and the bar says how far
        // into the range that was rather than printing the hop as if it were a
        // fraction of the sweep.
        view.apply_ttl(TtlEvent::Found(min + 4));
        assert_eq!(view.done, 5);
        assert_eq!(view.label(), format!("5 / {hops}"));
    }

    #[test]
    fn a_zero_total_does_not_panic() {
        // The sweep reports nothing at all before it has sized itself up, and
        // `Gauge::ratio` asserts on anything outside 0..=1.
        let mut view = ProgressView::new();
        view.apply(SweepEvent::Step { done: 0, total: 0 });
        view.apply(SweepEvent::Step { done: 99, total: 0 });
        assert_eq!(view.ratio(), 0.0);

        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| view.render(f, f.area())).unwrap();
    }

    /// The frame has to be repaintable on a timer with no new event at all:
    /// that is the whole point of the spinner running off the clock.
    #[test]
    fn repaints_without_any_new_event() {
        let mut view = ProgressView::new();
        let view = {
            view.apply(SweepEvent::Step { done: 5, total: 20 });
            view
        };
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| view.render(f, f.area())).unwrap();
        let first = format!("{:?}", terminal.backend().buffer());
        terminal.draw(|f| view.render(f, f.area())).unwrap();
        let second = format!("{:?}", terminal.backend().buffer());
        assert_eq!(first, second, "a repaint with no event changed nothing");
    }
}

