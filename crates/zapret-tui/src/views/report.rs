//! The autotune report: the same numbers the sweep produced, as tables.
//!
//! The console version of this was thirty `println!` calls of aligned-ish text.
//! Everything it showed is here, but split in two, because it answers two
//! different questions and they do not belong on one screen:
//!
//! - **Summary** — which strategy do I pick. A verdict, the ranking that
//!   produced it, and what the network is blocking.
//! - **Details** — why. Every preset, every domain, every protocol, and what
//!   each strategy did to each of them.

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;
use zapret_wrapper::autotune::{AutotuneResults, CheckStatus, DomainProtocolCheck, PresetResult, StrategyCheckResult};

use crate::state::screens::AutotuneReportTab;
use crate::theme::Theme;

use super::{fit, frame};

/// One column of a table: its heading, its width (padding included) and where
/// its text sits.
struct Column {
    title: String,
    width: u16,
    align: Alignment,
}

impl Column {
    fn new(title: String, width: u16) -> Self {
        Self {
            title,
            width,
            align: Alignment::Left,
        }
    }

    fn right(title: String, width: u16) -> Self {
        Self {
            title,
            width,
            align: Alignment::Right,
        }
    }
}

type Cell = (String, Style);

/// A rule is one cell wide; a table that measured it wrong would be ragged.
const RULE: Style = Style::new().fg(Color::DarkGray);
const EDGE: Style = Style::new().fg(Color::Yellow);
/// The width the flexible columns share, after the fixed ones and the vertical
/// rules have taken their share of the terminal.
fn room(reserved: u16, columns: u16, width: u16) -> u16 {
    // Six is the narrowest a column can be and still hold a status mark.
    (width.saturating_sub(reserved + columns + 1)).max(6)
}

/// Split what is left between two columns that both want to grow.
fn room2(reserved: u16, columns: u16, width: u16) -> (u16, u16) {
    let free = room(reserved, columns, width);
    (free / 2, free - free / 2)
}

fn mark(ok: bool) -> Cell {
    if ok {
        ("✅".to_string(), Theme::ok())
    } else {
        ("❌".to_string(), Theme::bad())
    }
}

fn plain(text: impl Into<String>) -> Cell {
    (text.into(), Style::default().fg(Color::White))
}

fn muted(text: impl Into<String>) -> Cell {
    (text.into(), Style::default().fg(Color::DarkGray))
}

fn verdict(status: &CheckStatus) -> Cell {
    match status {
        CheckStatus::Pass => (rust_i18n::t!("autotune_result_pass").into_owned(), Theme::ok()),
        CheckStatus::Fail => (rust_i18n::t!("autotune_result_fail").into_owned(), Theme::bad()),
        CheckStatus::Skip => (rust_i18n::t!("autotune_result_skip").into_owned(), Theme::warn()),
        CheckStatus::Error => (rust_i18n::t!("autotune_result_error").into_owned(), Theme::bad()),
    }
}

/// Fit `text` into `width` columns, measuring what the terminal will actually
/// draw rather than how many code points the string has.
fn pad(text: &str, width: u16, align: Alignment) -> String {
    let slack = width.saturating_sub(text.width() as u16);
    match align {
        Alignment::Left => format!("{}{}", text, " ".repeat(slack as usize)),
        Alignment::Right => format!("{}{}", " ".repeat(slack as usize), text),
        _ => {
            let left = slack / 2;
            format!(
                "{}{}{}",
                " ".repeat(left as usize),
                text,
                " ".repeat((slack - left) as usize)
            )
        }
    }
}

fn rule(left: char, joint: char, right: char, cols: &[Column]) -> Line<'static> {
    let mut line = String::new();
    line.push(left);
    for (i, col) in cols.iter().enumerate() {
        if i > 0 {
            line.push(joint);
        }
        line.push_str(&"─".repeat(col.width as usize));
    }
    line.push(right);
    Line::from(Span::styled(line, RULE))
}

fn row(cols: &[Column], cells: &[Cell], header: bool) -> Line<'static> {
    let mut spans = vec![Span::styled("│", RULE)];
    for (i, col) in cols.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("│", RULE));
        }
        let (text, style) = &cells[i];
        let style = if header { Theme::table_header() } else { *style };
        let inner = col.width - 2;
        spans.push(Span::styled(
            format!(" {} ", pad(&fit(text, inner), inner, col.align)),
            style,
        ));
    }
    spans.push(Span::styled("│", RULE));
    Line::from(spans)
}

/// A captioned table: caption, top rule, headings, rule, body, bottom rule.
fn table(caption: String, cols: &[Column], rows: Vec<Vec<Cell>>) -> Vec<Line<'static>> {
    let headings: Vec<Cell> = cols.iter().map(|c| (c.title.clone(), Style::default())).collect();

    let mut lines = vec![
        Line::from(Span::styled(format!(" {}", caption), Theme::title())),
        rule('┌', '┬', '┐', cols),
        row(cols, &headings, true),
        rule('├', '┼', '┤', cols),
    ];
    lines.extend(rows.into_iter().map(|cells| row(cols, &cells, false)));
    lines.push(rule('└', '┴', '┘', cols));
    lines.push(Line::from(""));
    lines
}

// ---------------------------------------------------------------- summary --

/// One strategy's showing across every preset it was tested on.
///
/// A strategy is not a row of one preset's table: the question the user leaves
/// the sweep with is "which one do I pick", and that needs every preset's
/// result for it in one place. `stable` is the part `works` cannot see — a
/// strategy that unblocks a domain over exactly one protocol works right now
/// and stops working the day that protocol is blocked.
struct Ranked {
    name: String,
    unblocked: usize,
    tested: usize,
    stable: usize,
    presets_passed: usize,
    presets_tested: usize,
    protocols: Vec<&'static str>,
}

impl Ranked {
    /// Everything at once, on every preset it was tried on.
    fn works_everywhere(&self) -> bool {
        self.presets_tested > 0 && self.presets_passed == self.presets_tested
    }

    /// Whether it lifts every domain it was pointed at.
    fn lifts_all(&self) -> bool {
        self.tested > 0 && self.unblocked == self.tested
    }

    fn protocol_list(&self) -> String {
        if self.protocols.is_empty() {
            rust_i18n::t!("atv_none").into_owned()
        } else {
            self.protocols.join(" ")
        }
    }
}

/// The sweep's four protocols, in the order a reader expects them, under the
/// names the wrapper uses.
const PROTOCOLS: [(&str, &str); 4] = [("HTTP", "HTTP"), ("TLS12", "T1.2"), ("TLS13", "T1.3"), ("QUIC", "QUIC")];

/// How many of the four transports a domain came through on.
fn breadth(dc: &DomainProtocolCheck) -> usize {
    [dc.http, dc.tls12, dc.tls13, dc.quic].iter().filter(|ok| **ok).count()
}

/// Merge every preset's results for each strategy, best first.
///
/// The order is the answer to the question: a strategy that holds up on every
/// preset beats one that rescues a single domain somewhere, and breadth breaks
/// the tie between two that cover the same ground.
fn ranking(results: &AutotuneResults) -> Vec<Ranked> {
    let mut ranked: Vec<Ranked> = Vec::new();

    for preset in &results.preset_results {
        for sr in &preset.strategy_results {
            // A domain that opens on more than one protocol is one that will
            // still be open if the other one gets blocked later.
            let stable = sr
                .domain_checks
                .iter()
                .filter(|dc| (dc.tls12 || dc.tls13) && breadth(dc) >= 2)
                .count();

            let entry = match ranked.iter_mut().find(|r| r.name == sr.strategy_name) {
                Some(entry) => entry,
                None => {
                    ranked.push(Ranked {
                        name: sr.strategy_name.clone(),
                        unblocked: 0,
                        tested: 0,
                        stable: 0,
                        presets_passed: 0,
                        presets_tested: 0,
                        protocols: Vec::new(),
                    });
                    ranked.last_mut().expect("just pushed")
                }
            };

            entry.unblocked += sr.score();
            entry.tested += sr.total();
            entry.stable += stable;
            entry.presets_tested += 1;
            if sr.works {
                entry.presets_passed += 1;
            }
            for (key, label) in PROTOCOLS {
                if sr.protocols_working.iter().any(|p| p == key) && !entry.protocols.contains(&label) {
                    entry.protocols.push(label);
                }
            }
        }
    }

    ranked.sort_by(|a, b| {
        b.presets_passed
            .cmp(&a.presets_passed)
            .then(b.unblocked.cmp(&a.unblocked))
            .then(b.stable.cmp(&a.stable))
            .then(b.protocols.len().cmp(&a.protocols.len()))
            .then(a.name.cmp(&b.name))
    });
    ranked
}

/// A short label for how much a strategy is worth, given that it is on this row.
fn verdict_for(entry: &Ranked, is_best: bool) -> Cell {
    if is_best {
        (rust_i18n::t!("atv_verdict_best").into_owned(), Theme::accent())
    } else if entry.lifts_all() && entry.works_everywhere() {
        (rust_i18n::t!("atv_verdict_stable").into_owned(), Theme::ok())
    } else if entry.unblocked > 0 {
        (rust_i18n::t!("atv_verdict_partial").into_owned(), Theme::warn())
    } else {
        (rust_i18n::t!("atv_verdict_none").into_owned(), Theme::bad())
    }
}

/// The one line the whole sweep exists to produce: which strategy to pick.
fn callout(entries: &[Ranked], width: u16) -> Vec<Line<'static>> {
    let best = entries.iter().find(|e| e.unblocked > 0);
    let title = match best {
        Some(best) => best.name.clone(),
        None => rust_i18n::t!("atv_no_winner").into_owned(),
    };

    let body: Vec<Cell> = match best {
        Some(best) => vec![
            muted(
                rust_i18n::t!("atv_best_unblocks")
                    .replace("{total}", &best.tested.to_string())
                    .replace("{stable}", &best.stable.to_string())
                    .replace("{}", &best.unblocked.to_string()),
            ),
            muted(rust_i18n::t!("atv_best_protocols").replace("{}", &best.protocol_list())),
            muted(
                rust_i18n::t!("atv_best_presets")
                    .replace("{total}", &best.presets_tested.to_string())
                    .replace("{}", &best.presets_passed.to_string()),
            ),
        ],
        None => vec![muted(rust_i18n::t!("atv_nothing_worked"))],
    };

    callout_box(title, &body, width)
}

/// A boxed aside, drawn into the same run of lines as everything else so it
/// scrolls with the report instead of floating above it.
fn callout_box(title: String, body: &[Cell], width: u16) -> Vec<Line<'static>> {
    let inner = width.saturating_sub(2);

    // "┏━ " is three cells, and the closing corner is one more, so the rule
    // between them is whatever is left over.
    let used = 3 + title.width() as u16;
    let mut lines = vec![Line::from(vec![
        Span::styled("┏━ ", EDGE),
        Span::styled(fit(&title, inner.saturating_sub(1)), Theme::accent()),
        Span::styled(
            format!("{}┓", "━".repeat(width.saturating_sub(used + 1) as usize)),
            EDGE,
        ),
    ])];

    for cell in body {
        lines.push(Line::from(vec![
            Span::styled("┃ ", EDGE),
            Span::styled(fit(&cell.0, inner), cell.1),
        ]));
    }
    lines.push(Line::from(Span::styled(
        format!("┗{}┛", "━".repeat(width.saturating_sub(2) as usize)),
        EDGE,
    )));
    lines.push(Line::from(""));
    lines
}

fn ranking_table(entries: &[Ranked], width: u16) -> Vec<Line<'static>> {
    // Wide enough for their own headings, so the table never has to ellipsise
    // "Unblocked" into "Unblock…".
    let fixed = 11 + 9 + 9 + 11;
    let (name_w, protocols_w) = room2(fixed, 6, width);
    let cols = vec![
        Column::new(rust_i18n::t!("atv_col_strategy").to_string(), name_w),
        Column::right(rust_i18n::t!("atv_col_unblocked").to_string(), 11),
        Column::right(rust_i18n::t!("atv_col_stable").to_string(), 9),
        Column::new(rust_i18n::t!("atv_col_protocols").to_string(), protocols_w),
        Column::right(rust_i18n::t!("atv_col_presets").to_string(), 9),
        Column::new(rust_i18n::t!("atv_col_verdict").to_string(), 11),
    ];

    let best_index = entries.iter().position(|e| e.unblocked > 0);
    let rows = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_best = Some(i) == best_index;
            let name_style = if is_best {
                Theme::accent()
            } else if entry.unblocked == 0 {
                Theme::bad()
            } else {
                Theme::ok()
            };
            vec![
                (entry.name.clone(), name_style),
                plain(format!("{}/{}", entry.unblocked, entry.tested)),
                plain(format!("{}/{}", entry.stable, entry.tested)),
                muted(entry.protocol_list()),
                plain(format!("{}/{}", entry.presets_passed, entry.presets_tested)),
                verdict_for(entry, is_best),
            ]
        })
        .collect();

    table(rust_i18n::t!("atv_section_rank").to_string(), &cols, rows)
}

fn net_table(results: &AutotuneResults, width: u16) -> Vec<Line<'static>> {
    let check_w = 12;
    let verdict_w = 14;
    let cols = vec![
        Column::new(rust_i18n::t!("atv_col_check").to_string(), check_w),
        Column::new(rust_i18n::t!("atv_col_verdict").to_string(), verdict_w),
        Column::new(
            rust_i18n::t!("atv_col_detail").to_string(),
            room(check_w + verdict_w, 3, width),
        ),
    ];

    let names = ["DNS", "TCP RST", "SNI", "SIBERIAN", "QUIC", "CIDR"];
    let rows = names
        .iter()
        .zip(&results.block_results)
        .map(|(name, check)| vec![plain(*name), verdict(&check.status), muted(check.detail.clone())])
        .collect();

    table(rust_i18n::t!("atv_section_net").to_string(), &cols, rows)
}

/// What "pick this one" looks like, and the ranking that produced it.
fn build_summary(results: &AutotuneResults, width: u16) -> Vec<Line<'static>> {
    let ranked = ranking(results);
    let mut lines = vec![
        Line::from(Span::styled(
            format!(
                "{}   {}",
                rust_i18n::t!("atv_elapsed_total").replace("{}", &clock(results.elapsed_secs)),
                rust_i18n::t!("atv_legend")
            ),
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
    ];

    // The verdict comes before anything else: it is what the sweep was run for.
    if ranked.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" {}", rust_i18n::t!("atv_no_strategies")),
            Theme::warn(),
        )));
        lines.push(Line::from(""));
    } else {
        lines.extend(callout(&ranked, width));
        lines.extend(ranking_table(&ranked, width));
    }

    lines.extend(net_table(results, width));

    lines.push(Line::from(Span::styled(
        format!(" {}", rust_i18n::t!("atv_section_common")),
        Theme::title(),
    )));
    lines.push(Line::from(""));
    if results.common_strategies.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("   "),
            Span::styled(rust_i18n::t!("atv_none"), Theme::warn()),
        ]));
    } else {
        for name in &results.common_strategies {
            lines.push(Line::from(vec![
                Span::styled("   ✅ ", Theme::ok()),
                Span::styled(name.clone(), Style::default().fg(Color::White)),
            ]));
        }
    }
    lines
}

// ---------------------------------------------------------------- details --

fn domain_table(preset: &PresetResult, width: u16) -> Vec<Line<'static>> {
    let fixed = 6 + 7 + 7 + 6 + 12;
    let cols = vec![
        Column::new(rust_i18n::t!("atv_col_domain").to_string(), room(fixed, 6, width)),
        Column::right(rust_i18n::t!("atv_col_http").to_string(), 6),
        Column::right(rust_i18n::t!("atv_col_tls12").to_string(), 7),
        Column::right(rust_i18n::t!("atv_col_tls13").to_string(), 7),
        Column::right(rust_i18n::t!("atv_col_quic").to_string(), 6),
        Column::new(rust_i18n::t!("atv_col_state").to_string(), 12),
    ];

    let rows = preset
        .domain_checks
        .iter()
        .map(|dc| {
            let state = if dc.baseline_pass {
                (rust_i18n::t!("atv_open").to_string(), Theme::ok())
            } else {
                (rust_i18n::t!("atv_blocked_word").to_string(), Theme::bad())
            };
            vec![
                plain(dc.domain.clone()),
                mark(dc.http == CheckStatus::Pass),
                mark(dc.tls12 == CheckStatus::Pass),
                mark(dc.tls13 == CheckStatus::Pass),
                mark(dc.quic == CheckStatus::Pass),
                state,
            ]
        })
        .collect();

    table(rust_i18n::t!("atv_section_domains").to_string(), &cols, rows)
}

fn strategy_cell(sr: &StrategyCheckResult) -> Cell {
    let style = if sr.works { Theme::ok() } else { Theme::bad() };
    (sr.strategy_name.clone(), style)
}

fn strategy_table(preset: &PresetResult, width: u16) -> Vec<Line<'static>> {
    let fixed = 7 + 6 + 7 + 7 + 6;
    let (strategy_w, blocked_w) = room2(fixed, 7, width);
    let cols = vec![
        Column::new(rust_i18n::t!("atv_col_strategy").to_string(), strategy_w),
        Column::right(rust_i18n::t!("atv_col_score").to_string(), 7),
        Column::right(rust_i18n::t!("atv_col_http").to_string(), 6),
        Column::right(rust_i18n::t!("atv_col_tls12").to_string(), 7),
        Column::right(rust_i18n::t!("atv_col_tls13").to_string(), 7),
        Column::right(rust_i18n::t!("atv_col_quic").to_string(), 6),
        Column::new(rust_i18n::t!("atv_col_blocked").to_string(), blocked_w),
    ];

    let rows = preset
        .strategy_results
        .iter()
        .map(|sr| {
            let has = |proto: &str| sr.protocols_working.iter().any(|p| p == proto);
            let blocked = if sr.domains_fail.is_empty() {
                muted(rust_i18n::t!("atv_none").to_string())
            } else {
                muted(sr.domains_fail.join(", "))
            };
            vec![
                strategy_cell(sr),
                plain(format!("{}/{}", sr.score(), sr.total())),
                mark(has("HTTP")),
                mark(has("TLS12")),
                mark(has("TLS13")),
                mark(has("QUIC")),
                blocked,
            ]
        })
        .collect();

    table(rust_i18n::t!("atv_section_strategies").to_string(), &cols, rows)
}

/// What one strategy did to each domain it was pointed at.
///
/// This is the row-by-row evidence behind the aggregate marks above, and it is
/// the only place the per-domain protocol results show up at all.
fn domain_breakdown(sr: &StrategyCheckResult, width: u16) -> Vec<Line<'static>> {
    // The four protocol columns, counted with the padding that goes with them.
    let fixed = 7 + 7 + 7 + 6;
    let cols = vec![
        Column::new(rust_i18n::t!("atv_col_domain").to_string(), room(fixed, 5, width)),
        Column::right(rust_i18n::t!("atv_col_http").to_string(), 7),
        Column::right(rust_i18n::t!("atv_col_tls12").to_string(), 7),
        Column::right(rust_i18n::t!("atv_col_tls13").to_string(), 7),
        Column::right(rust_i18n::t!("atv_col_quic").to_string(), 6),
    ];

    let rows = sr
        .domain_checks
        .iter()
        .map(|dc| {
            // Browsers use HTTPS, so plain HTTP on port 80 is not an opening.
            let opens = dc.tls12 || dc.tls13;
            let style = if opens && breadth(dc) >= 2 {
                Theme::ok()
            } else if opens {
                Theme::warn()
            } else {
                Theme::bad()
            };
            vec![
                (dc.domain.clone(), style),
                mark(dc.http),
                mark(dc.tls12),
                mark(dc.tls13),
                mark(dc.quic),
            ]
        })
        .collect();

    table(
        rust_i18n::t!("atv_section_detail").replace("{}", &sr.strategy_name),
        &cols,
        rows,
    )
}

/// The long way round: every preset, every domain, every protocol.
fn build_details(results: &AutotuneResults, width: u16) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for preset in &results.preset_results {
        let working = preset.strategy_results.iter().filter(|s| s.works).count();
        lines.push(Line::from(Span::styled(
            format!(
                "{}   {}",
                rust_i18n::t!("atv_preset_heading").replace("{}", &preset.preset_name),
                rust_i18n::t!("atv_strat_tally")
                    .replace("{total}", &preset.strategy_results.len().to_string())
                    .replace("{}", &working.to_string())
            ),
            Theme::accent(),
        )));
        lines.push(Line::from(""));

        lines.extend(domain_table(preset, width));
        if !preset.strategy_results.is_empty() {
            lines.extend(strategy_table(preset, width));
            for sr in &preset.strategy_results {
                if !sr.domain_checks.is_empty() {
                    lines.extend(domain_breakdown(sr, width));
                }
            }
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" {}", rust_i18n::t!("atv_no_results")),
            Theme::warn(),
        )));
    }
    lines
}

// ------------------------------------------------------------------ shell --

/// The tab strip on the frame's top border, with the open one inverted.
fn tab_title(active: AutotuneReportTab) -> Line<'static> {
    let mut spans = vec![Span::raw(" ")];
    for (i, tab) in AutotuneReportTab::ALL.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", RULE));
        }
        let label = tab.label();
        spans.push(if *tab == active {
            Span::styled(label, Theme::table_header())
        } else {
            Span::styled(label, Style::default().fg(Color::Gray))
        });
    }
    Line::from(spans)
}

/// The whole report as one run of lines, ready to be scrolled.
///
/// `width` is the inner width of the report area; the flexible columns size
/// themselves against it, and anything that still does not fit is cut rather
/// than wrapped, so a row always stays a row.
pub fn build(results: &AutotuneResults, tab: AutotuneReportTab, width: u16) -> Vec<Line<'static>> {
    match tab {
        AutotuneReportTab::Summary => build_summary(results, width),
        AutotuneReportTab::Details => build_details(results, width),
    }
}

/// Paint the report and pull `scroll` back into range.
///
/// The clamp writes through because either tab grows taller than any terminal,
/// and a scroll offset that is never pulled back makes the view look stuck at
/// the bottom.
pub fn render(f: &mut Frame, area: Rect, results: &AutotuneResults, tab: AutotuneReportTab, scroll: &mut usize) {
    let block = frame(None).title(tab_title(tab));
    let inner = block.inner(area);
    let lines = build(results, tab, inner.width);

    let max_scroll = (lines.len() as u16).saturating_sub(inner.height) as usize;
    if *scroll > max_scroll {
        *scroll = max_scroll;
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll(((*scroll).min(u16::MAX as usize) as u16, 0));
    f.render_widget(paragraph, area);
}

/// `mm:ss`, matching the progress screen.
fn clock(secs: u64) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use zapret_wrapper::autotune::{CheckResult, DomainCheckResult};

    fn dc(domain: &str, http: bool, tls12: bool, tls13: bool, quic: bool) -> DomainProtocolCheck {
        DomainProtocolCheck {
            domain: domain.to_string(),
            http,
            tls12,
            tls13,
            quic,
        }
    }

    fn strat(
        name: &str,
        passed: &[&str],
        failed: &[&str],
        protocols: &[&str],
        checks: Vec<DomainProtocolCheck>,
    ) -> StrategyCheckResult {
        let works = passed.len() >= failed.len();
        StrategyCheckResult {
            strategy_name: name.to_string(),
            domains_pass: passed.iter().map(|d| d.to_string()).collect(),
            domains_fail: failed.iter().map(|d| d.to_string()).collect(),
            works,
            protocols_working: protocols.iter().map(|p| p.to_string()).collect(),
            domain_checks: checks,
        }
    }

    fn blocked_domain(name: &str) -> DomainCheckResult {
        DomainCheckResult {
            domain: name.to_string(),
            alive: CheckStatus::Pass,
            http: CheckStatus::Pass,
            tls12: CheckStatus::Pass,
            tls13: CheckStatus::Fail,
            quic: CheckStatus::Fail,
            baseline_pass: false,
            detail: String::new(),
            http_count: 0,
            quic_count: 0,
        }
    }

    /// Two presets and four strategies, shaped so the ranking has something to
    /// decide: one that holds up everywhere, one that covers the same ground
    /// more narrowly, one that only rescues a single preset, and one that does
    /// nothing at all.
    pub(super) fn sample() -> AutotuneResults {
        AutotuneResults {
            block_results: vec![
                CheckResult::fail("Possible DNS spoofing: discord.com resolved to sinkhole IPs: [0.0.0.0]"),
                CheckResult::pass("TCP connections successful, no RST detected"),
                CheckResult::skip("Not selected"),
            ],
            preset_results: vec![
                PresetResult {
                    preset_name: "Discord".into(),
                    domain_checks: vec![blocked_domain("discord.com")],
                    strategy_results: vec![
                        strat(
                            "tls13_autotls",
                            &["discord.com", "discordapp.com"],
                            &[],
                            &["HTTP", "TLS12", "TLS13"],
                            vec![
                                dc("discord.com", true, true, true, false),
                                dc("discordapp.com", true, true, true, true),
                            ],
                        ),
                        strat(
                            "general (ALT11_setting)",
                            &["discord.com", "discordapp.com"],
                            &[],
                            &["HTTP", "TLS13"],
                            vec![
                                dc("discord.com", true, false, true, false),
                                dc("discordapp.com", false, false, true, false),
                            ],
                        ),
                        strat("dilation", &["discord.com"], &[], &["HTTP", "TLS12"], Vec::new()),
                        strat("fake", &[], &["discord.com", "discordapp.com"], &[], Vec::new()),
                    ],
                },
                PresetResult {
                    preset_name: "YouTube".into(),
                    domain_checks: vec![blocked_domain("youtube.com")],
                    strategy_results: vec![
                        strat(
                            "tls13_autotls",
                            &["youtube.com", "googlevideo.com"],
                            &[],
                            &["HTTP", "TLS12", "TLS13", "QUIC"],
                            vec![
                                dc("youtube.com", true, true, true, true),
                                dc("googlevideo.com", true, true, true, false),
                            ],
                        ),
                        strat(
                            "general (ALT11_setting)",
                            &["youtube.com"],
                            &["googlevideo.com"],
                            &["HTTP", "TLS13"],
                            vec![dc("youtube.com", true, false, true, false)],
                        ),
                        strat("dilation", &[], &["youtube.com", "googlevideo.com"], &[], Vec::new()),
                        strat("fake", &[], &["youtube.com", "googlevideo.com"], &[], Vec::new()),
                    ],
                },
            ],
            common_strategies: vec!["tls13_autotls".into()],
            elapsed_secs: 754,
        }
    }

    /// The one thing the whole report exists to answer. If the order ever puts
    /// the single-preset rescue above the one that holds up everywhere, the
    /// report is actively misleading.
    #[test]
    fn the_most_stable_strategy_comes_first() {
        let ranked = ranking(&sample());
        let names: Vec<&str> = ranked.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["tls13_autotls", "general (ALT11_setting)", "dilation", "fake"]
        );

        let best = &ranked[0];
        assert_eq!((best.unblocked, best.tested), (4, 4));
        assert_eq!((best.presets_passed, best.presets_tested), (2, 2));
        assert_eq!(best.protocols, vec!["HTTP", "T1.2", "T1.3", "QUIC"]);

        // Every domain the best one lifts opens on more than one protocol,
        // which is what separates it from the strategy that lifts the same
        // domains over TLS 1.3 alone — that one is one blocked protocol away
        // from being useless.
        assert_eq!(best.stable, 4);
        assert_eq!(ranked[1].stable, 2);
    }

    #[test]
    fn a_domain_that_opens_on_one_protocol_is_not_counted_as_stable() {
        let results = AutotuneResults {
            block_results: Vec::new(),
            preset_results: vec![PresetResult {
                preset_name: "One".into(),
                domain_checks: Vec::new(),
                strategy_results: vec![
                    strat(
                        "narrow",
                        &["a.com"],
                        &[],
                        &["TLS13"],
                        vec![dc("a.com", false, false, true, false)],
                    ),
                    strat(
                        "wide",
                        &["a.com"],
                        &[],
                        &["TLS12", "TLS13"],
                        vec![dc("a.com", false, true, true, false)],
                    ),
                ],
            }],
            common_strategies: Vec::new(),
            elapsed_secs: 1,
        };

        let ranked = ranking(&results);
        assert_eq!(ranked[0].name, "wide");
        assert_eq!(ranked[0].stable, 1);
        assert_eq!(ranked[1].stable, 0);
    }

    /// The whole point of a table view is that a row is a row: every boxed line
    /// has to come out exactly as wide as the area it was given, or the frame is
    /// ragged, a column is cut off, or a rule runs into the border.
    #[test]
    fn every_boxed_line_is_exactly_the_area_width() {
        for tab in AutotuneReportTab::ALL {
            for width in [60u16, 80, 100, 140] {
                let lines = build(&sample(), tab, width);
                let boxed: Vec<(usize, usize)> = lines
                    .iter()
                    .filter(|l| {
                        l.spans
                            .first()
                            .map(|s| matches!(s.content.as_ref(), "│" | "┏" | "┃" | "┗"))
                            .unwrap_or(false)
                    })
                    .map(|l| (l.width(), y_of(&lines, l)))
                    .collect();
                assert!(!boxed.is_empty(), "no boxed lines in {tab:?} at {width}");
                for (got, y) in &boxed {
                    assert_eq!(*got, width as usize, "line {y} in {tab:?} is {got} wide, not {width}");
                }
            }
        }
    }

    /// Position of `line` in `lines`, for a failure message that has to name one.
    fn y_of(lines: &[Line<'_>], line: &Line<'_>) -> usize {
        lines.iter().position(|l| std::ptr::eq(l, line)).unwrap_or(0)
    }

    #[test]
    fn scrolling_past_the_end_shows_the_last_line() {
        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        let results = sample();
        let mut scroll = 9999;
        terminal
            .draw(|f| render(f, f.area(), &results, AutotuneReportTab::Details, &mut scroll))
            .unwrap();

        let lines = build(&results, AutotuneReportTab::Details, 58);
        assert_eq!(scroll, lines.len() - 10, "scroll was not pulled back");
    }
}
