use std::borrow::Cow;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

use super::release_notes::release_notes_close_button_rect;
use super::scrollbar::{release_notes_scrollbar_rect, render_scrollbar};
use super::widgets::{
    modal_stack_areas, panel_contrast_fg, render_action_button, render_modal_header,
    render_modal_shell,
};
use crate::app::{AppState, NavigateAction};

/// An action that can be triggered by clicking a row in the keybinds overlay.
///
/// Every clickable entry in the keybinds list carries one of these so a single
/// mouse click can run whatever that binding does — most usefully, a custom
/// command that the user has configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeybindHelpAction {
    /// A built-in action, dispatched exactly as if its keybind were pressed.
    Navigate(NavigateAction),
    /// A user-defined custom command, identified by its index in
    /// `keybinds.custom_commands`.
    CustomCommand(usize),
}

/// A single rendered row of the keybinds overlay.
///
/// `width` is the display width of the row's text (used for wrap-aware scroll
/// math and click hit-testing); `action` is `Some` when the row is clickable.
pub(crate) struct KeybindHelpRow {
    pub width: usize,
    pub line: Line<'static>,
    pub action: Option<KeybindHelpAction>,
}

pub(super) type HelpEntry = (String, Cow<'static, str>, Option<KeybindHelpAction>);
pub(super) type HelpGroup = (&'static str, Vec<HelpEntry>);

// Build a clickable help entry whose click runs the given built-in action.
fn entry(key: impl Into<String>, label: &'static str, action: NavigateAction) -> HelpEntry {
    (
        key.into(),
        Cow::Borrowed(label),
        Some(KeybindHelpAction::Navigate(action)),
    )
}

// Build an informational (non-clickable) help entry. Used for keys whose
// behavior is purely contextual (movement, indexed switches) and so cannot be
// meaningfully invoked with a single click.
fn doc_entry(key: impl Into<String>, label: &'static str) -> HelpEntry {
    (key.into(), Cow::Borrowed(label), None)
}

fn keybind_label(bindings: &crate::config::ActionKeybinds) -> String {
    bindings.label().unwrap_or_else(|| "unset".to_string())
}

fn indexed_label(bindings: &[crate::config::IndexedKeybind]) -> String {
    if bindings.is_empty() {
        "unset".to_string()
    } else if bindings.len() == 9 {
        let first = &bindings[0].label;
        if first.ends_with('1') {
            format!("{}1..9", first.trim_end_matches('1'))
        } else {
            bindings
                .iter()
                .map(|binding| binding.label.clone())
                .collect::<Vec<_>>()
                .join(" / ")
        }
    } else {
        bindings
            .iter()
            .map(|binding| binding.label.clone())
            .collect::<Vec<_>>()
            .join(" / ")
    }
}

pub(super) fn keybind_help_groups(app: &AppState) -> Vec<HelpGroup> {
    let kb = &app.keybinds;
    let mut groups = Vec::new();

    groups.push((
        "global",
        vec![
            doc_entry(
                crate::config::format_key_combo((app.prefix_code, app.prefix_mods)),
                "prefix mode",
            ),
            doc_entry(keybind_label(&kb.help), "keybinds"),
            entry(
                keybind_label(&kb.settings),
                "settings",
                NavigateAction::Settings,
            ),
            entry(keybind_label(&kb.detach), "detach", NavigateAction::Detach),
            entry(
                keybind_label(&kb.reload_config),
                "reload config",
                NavigateAction::ReloadConfig,
            ),
            entry(
                keybind_label(&kb.open_notification_target),
                "open notification target",
                NavigateAction::OpenNotificationTarget,
            ),
        ],
    ));

    groups.push((
        "navigation",
        vec![
            doc_entry("esc", "back"),
            doc_entry(
                format!(
                    "{} / {}",
                    keybind_label(&kb.navigate.workspace_up),
                    keybind_label(&kb.navigate.workspace_down)
                ),
                "workspace list",
            ),
            doc_entry(
                format!(
                    "{} / {} / {} / {} / left / right",
                    keybind_label(&kb.navigate.pane_left),
                    keybind_label(&kb.navigate.pane_down),
                    keybind_label(&kb.navigate.pane_up),
                    keybind_label(&kb.navigate.pane_right)
                ),
                "move focus",
            ),
            doc_entry("tab / shift+tab", "cycle pane"),
            doc_entry("enter", "open workspace"),
            doc_entry("1..9", "switch workspace"),
        ],
    ));

    let workspace_tab = vec![
        entry(
            keybind_label(&kb.workspace_picker),
            "workspace navigation",
            NavigateAction::WorkspacePicker,
        ),
        entry(
            keybind_label(&kb.goto),
            "session navigator",
            NavigateAction::OpenNavigator,
        ),
        entry(
            keybind_label(&kb.new_workspace),
            "new workspace",
            NavigateAction::NewWorkspace,
        ),
        entry(
            keybind_label(&kb.new_worktree),
            "new worktree",
            NavigateAction::NewWorktree,
        ),
        entry(
            keybind_label(&kb.open_worktree),
            "open worktree",
            NavigateAction::OpenWorktree,
        ),
        entry(
            keybind_label(&kb.remove_worktree),
            "delete worktree checkout",
            NavigateAction::RemoveWorktree,
        ),
        entry(
            keybind_label(&kb.rename_workspace),
            "rename workspace",
            NavigateAction::RenameWorkspace,
        ),
        entry(
            keybind_label(&kb.close_workspace),
            "close workspace",
            NavigateAction::CloseWorkspace,
        ),
        entry(
            keybind_label(&kb.previous_workspace),
            "previous workspace",
            NavigateAction::PreviousWorkspace,
        ),
        entry(
            keybind_label(&kb.next_workspace),
            "next workspace",
            NavigateAction::NextWorkspace,
        ),
        doc_entry(indexed_label(&kb.switch_workspace), "switch workspace 1-9"),
        entry(
            keybind_label(&kb.previous_agent),
            "previous agent",
            NavigateAction::PreviousAgent,
        ),
        entry(
            keybind_label(&kb.next_agent),
            "next agent",
            NavigateAction::NextAgent,
        ),
        doc_entry(indexed_label(&kb.focus_agent), "focus agent 1-9"),
        entry(
            keybind_label(&kb.new_tab),
            "new tab",
            NavigateAction::NewTab,
        ),
        entry(
            keybind_label(&kb.rename_tab),
            "rename tab",
            NavigateAction::RenameTab,
        ),
        entry(
            keybind_label(&kb.previous_tab),
            "previous tab",
            NavigateAction::PreviousTab,
        ),
        entry(
            keybind_label(&kb.next_tab),
            "next tab",
            NavigateAction::NextTab,
        ),
        doc_entry(indexed_label(&kb.switch_tab), "switch tab 1-9"),
        entry(
            keybind_label(&kb.close_tab),
            "close tab",
            NavigateAction::CloseTab,
        ),
    ];
    groups.push(("workspaces / tabs", workspace_tab));

    let panes = vec![
        entry(
            keybind_label(&kb.split_vertical),
            "split vertical",
            NavigateAction::SplitVertical,
        ),
        entry(
            keybind_label(&kb.split_horizontal),
            "split horizontal",
            NavigateAction::SplitHorizontal,
        ),
        entry(
            keybind_label(&kb.close_pane),
            "close pane",
            NavigateAction::ClosePane,
        ),
        entry(
            keybind_label(&kb.rename_pane),
            "rename pane",
            NavigateAction::RenamePane,
        ),
        entry(
            keybind_label(&kb.edit_scrollback),
            "edit scrollback",
            NavigateAction::EditScrollback,
        ),
        entry(
            keybind_label(&kb.copy_mode),
            "copy mode",
            NavigateAction::CopyMode,
        ),
        entry(keybind_label(&kb.zoom), "zoom pane", NavigateAction::Zoom),
        entry(
            keybind_label(&kb.resize_mode),
            "resize mode",
            NavigateAction::EnterResizeMode,
        ),
        entry(
            keybind_label(&kb.toggle_sidebar),
            "toggle sidebar",
            NavigateAction::ToggleSidebar,
        ),
        entry(
            keybind_label(&kb.focus_pane_left),
            "focus pane left",
            NavigateAction::FocusPaneLeft,
        ),
        entry(
            keybind_label(&kb.focus_pane_down),
            "focus pane down",
            NavigateAction::FocusPaneDown,
        ),
        entry(
            keybind_label(&kb.focus_pane_up),
            "focus pane up",
            NavigateAction::FocusPaneUp,
        ),
        entry(
            keybind_label(&kb.focus_pane_right),
            "focus pane right",
            NavigateAction::FocusPaneRight,
        ),
        entry(
            keybind_label(&kb.cycle_pane_next),
            "cycle pane next",
            NavigateAction::CyclePaneNext,
        ),
        entry(
            keybind_label(&kb.cycle_pane_previous),
            "cycle pane previous",
            NavigateAction::CyclePanePrevious,
        ),
        entry(
            keybind_label(&kb.last_pane),
            "last pane",
            NavigateAction::LastPane,
        ),
    ];
    groups.push(("panes", panes));

    if !kb.custom_commands.is_empty() {
        groups.push((
            "custom",
            kb.custom_commands
                .iter()
                .enumerate()
                .map(|(idx, binding)| {
                    (
                        binding.label.clone(),
                        binding
                            .description
                            .clone()
                            .map(Cow::Owned)
                            .unwrap_or(Cow::Borrowed("custom command")),
                        Some(KeybindHelpAction::CustomCommand(idx)),
                    )
                })
                .collect(),
        ));
    }

    groups
}

pub(crate) fn keybind_help_rows(app: &AppState) -> Vec<KeybindHelpRow> {
    let heading_style = Style::default()
        .fg(app.palette.accent)
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default()
        .fg(app.palette.mauve)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(app.palette.text);

    let groups = keybind_help_groups(app);
    let key_width = groups
        .iter()
        .flat_map(|(_, entries)| entries.iter().map(|(key, _, _)| key.chars().count()))
        .max()
        .unwrap_or(8);

    let mut rows = Vec::new();

    for (group, entries) in groups {
        rows.push(KeybindHelpRow {
            width: group.len() + 1,
            line: Line::from(vec![Span::styled(format!(" {group}"), heading_style)]),
            action: None,
        });
        for (key, label, action) in entries {
            let padded_key = format!(" {:<width$} ", key, width = key_width);
            let width = padded_key.chars().count() + label.chars().count();
            rows.push(KeybindHelpRow {
                width,
                line: Line::from(vec![
                    Span::styled(padded_key, key_style),
                    Span::styled(label.into_owned(), label_style),
                ]),
                action,
            });
        }
        rows.push(KeybindHelpRow {
            width: 0,
            line: Line::raw(""),
            action: None,
        });
    }

    rows
}

// Re-style a clickable row to show the mouse-hover highlight: paint every span
// with the hover background and pad the rest of the row so the bar spans the
// full text column (only when the row fits on a single line).
fn highlight_hover_line(
    line: Line<'static>,
    width: usize,
    text_width: usize,
    bg: Color,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = line
        .spans
        .into_iter()
        .map(|span| {
            let style = span.style.bg(bg);
            Span::styled(span.content, style)
        })
        .collect();
    if width < text_width {
        spans.push(Span::styled(
            " ".repeat(text_width - width),
            Style::default().bg(bg),
        ));
    }
    Line::from(spans)
}

pub(super) fn render_keybind_help_overlay(app: &AppState, frame: &mut Frame) {
    super::dim_background(frame, frame.area());

    let Some(inner) = render_modal_shell(frame, frame.area(), 76, 22, &app.palette) else {
        return;
    };
    if inner.height < 6 || inner.width < 20 {
        return;
    }

    let stack = modal_stack_areas(inner, 2, 1, 0, 1);
    let header_rows =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas::<2>(stack.header);

    render_modal_header(frame, header_rows[0], "keybinds", &app.palette);
    render_action_button(
        frame,
        release_notes_close_button_rect(header_rows[0]),
        Some("esc"),
        "close",
        Style::default()
            .fg(panel_contrast_fg(&app.palette))
            .bg(app.palette.accent)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(
        Paragraph::new(" available commands and configured shortcuts")
            .style(Style::default().fg(app.palette.overlay1)),
        header_rows[1],
    );

    let body_area = stack.content;
    let metrics = crate::pane::ScrollMetrics {
        offset_from_bottom: app
            .keybind_help_max_scroll()
            .saturating_sub(app.keybind_help.scroll) as usize,
        max_offset_from_bottom: app.keybind_help_max_scroll() as usize,
        viewport_rows: body_area.height.max(1) as usize,
    };
    let track = release_notes_scrollbar_rect(body_area, metrics);
    let text_area = track
        .map(|_| {
            Rect::new(
                body_area.x,
                body_area.y,
                body_area.width.saturating_sub(1),
                body_area.height,
            )
        })
        .unwrap_or(body_area);

    let text_width = text_area.width as usize;
    let hovered = app.keybind_help.hovered;
    let hover_bg = app.palette.surface1;
    let lines = keybind_help_rows(app)
        .into_iter()
        .enumerate()
        .map(|(idx, row)| {
            if row.action.is_some() && Some(idx) == hovered {
                highlight_hover_line(row.line, row.width, text_width, hover_bg)
            } else {
                row.line
            }
        })
        .collect::<Vec<_>>();
    let body = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.keybind_help.scroll, 0));
    frame.render_widget(body, text_area);
    if let Some(track) = track {
        render_scrollbar(
            frame,
            metrics,
            track,
            app.palette.overlay0,
            app.palette.overlay1,
            "▐",
        );
    }

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" run ", Style::default().fg(app.palette.overlay0)),
            Span::styled("click row", Style::default().fg(app.palette.text)),
            Span::styled("  ·  ", Style::default().fg(app.palette.overlay0)),
            Span::styled("scroll", Style::default().fg(app.palette.overlay0)),
            Span::styled(" wheel ↑↓ ", Style::default().fg(app.palette.text)),
            Span::styled("  ·  ", Style::default().fg(app.palette.overlay0)),
            Span::styled("close", Style::default().fg(app.palette.overlay0)),
            Span::styled(" esc / enter ", Style::default().fg(app.palette.text)),
        ])),
        stack.footer.unwrap_or_default(),
    );
}
