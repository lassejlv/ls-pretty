use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs as RatatuiTabs},
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub name: String,
    pub path: PathBuf,
    pub content: String,
    pub original_content: String,
    pub has_unsaved_changes: bool,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub file_version: i32,
}

impl Tab {
    pub fn new(id: usize, name: String, path: PathBuf, content: String) -> Self {
        Self {
            id,
            name,
            path,
            content: content.clone(),
            original_content: content,
            has_unsaved_changes: false,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            file_version: 1,
        }
    }

    pub fn get_display_name(&self) -> String {
        if self.has_unsaved_changes {
            format!("{}*", self.name)
        } else {
            self.name.clone()
        }
    }

    pub fn mark_dirty(&mut self) {
        self.has_unsaved_changes = true;
    }

    pub fn mark_clean(&mut self) {
        self.has_unsaved_changes = false;
        self.original_content = self.content.clone();
    }

    pub fn is_dirty(&self) -> bool {
        self.has_unsaved_changes
    }

    pub fn revert_changes(&mut self) {
        self.content = self.original_content.clone();
        self.has_unsaved_changes = false;
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }
}

pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab: usize,
    next_id: usize,
    pub show_close_confirmation: bool,
    pub tab_to_close: Option<usize>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            next_id: 1,
            show_close_confirmation: false,
            tab_to_close: None,
        }
    }

    pub fn add_tab(&mut self, name: String, path: PathBuf, content: String) -> usize {
        // Check if file is already open
        if let Some(existing_tab_idx) = self.find_tab_by_path(&path) {
            self.active_tab = existing_tab_idx;
            return existing_tab_idx;
        }

        let id = self.next_id;
        self.next_id += 1;

        let tab = Tab::new(id, name, path, content);
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;

        self.active_tab
    }

    pub fn close_tab(&mut self, index: usize) -> Result<(), String> {
        if index >= self.tabs.len() {
            return Err("Tab index out of bounds".to_string());
        }

        if self.tabs[index].has_unsaved_changes {
            self.tab_to_close = Some(index);
            self.show_close_confirmation = true;
            return Err("Tab has unsaved changes".to_string());
        }

        self.tabs.remove(index);

        if self.tabs.is_empty() {
            self.active_tab = 0;
        } else if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        } else if index <= self.active_tab && self.active_tab > 0 {
            self.active_tab -= 1;
        }

        Ok(())
    }

    pub fn force_close_tab(&mut self, index: usize) -> Result<(), String> {
        if index >= self.tabs.len() {
            return Err("Tab index out of bounds".to_string());
        }

        self.tabs.remove(index);

        if self.tabs.is_empty() {
            self.active_tab = 0;
        } else if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        } else if index <= self.active_tab && self.active_tab > 0 {
            self.active_tab -= 1;
        }

        Ok(())
    }

    pub fn confirm_close_tab(&mut self) {
        if let Some(index) = self.tab_to_close {
            let _ = self.force_close_tab(index);
        }
        self.show_close_confirmation = false;
        self.tab_to_close = None;
    }

    pub fn cancel_close_tab(&mut self) {
        self.show_close_confirmation = false;
        self.tab_to_close = None;
    }

    pub fn close_active_tab(&mut self) -> Result<(), String> {
        if self.tabs.is_empty() {
            return Err("No tabs to close".to_string());
        }
        self.close_tab(self.active_tab)
    }

    pub fn switch_to_tab(&mut self, index: usize) -> Result<(), String> {
        if index >= self.tabs.len() {
            return Err("Tab index out of bounds".to_string());
        }
        self.active_tab = index;
        Ok(())
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    pub fn previous_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
        }
    }

    pub fn get_active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab)
    }

    pub fn get_active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab)
    }

    pub fn get_tab(&self, index: usize) -> Option<&Tab> {
        self.tabs.get(index)
    }

    pub fn get_tab_mut(&mut self, index: usize) -> Option<&mut Tab> {
        self.tabs.get_mut(index)
    }

    pub fn has_tabs(&self) -> bool {
        !self.tabs.is_empty()
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn get_active_tab_index(&self) -> usize {
        self.active_tab
    }

    pub fn find_tab_by_path(&self, path: &PathBuf) -> Option<usize> {
        self.tabs.iter().position(|tab| tab.path == *path)
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.tabs.iter().any(|tab| tab.has_unsaved_changes)
    }

    pub fn get_unsaved_tabs(&self) -> Vec<&Tab> {
        self.tabs
            .iter()
            .filter(|tab| tab.has_unsaved_changes)
            .collect()
    }

    pub fn save_active_tab(&mut self) -> Result<String, String> {
        if let Some(tab) = self.get_active_tab_mut() {
            tab.mark_clean();
            Ok(tab.content.clone())
        } else {
            Err("No active tab".to_string())
        }
    }

    pub fn save_all_tabs(&mut self) -> Vec<(PathBuf, String)> {
        let mut saved_files = Vec::new();
        for tab in &mut self.tabs {
            if tab.has_unsaved_changes {
                tab.mark_clean();
                saved_files.push((tab.path.clone(), tab.content.clone()));
            }
        }
        saved_files
    }

    pub fn render_tabs(&self, f: &mut Frame, area: Rect) {
        if self.tabs.is_empty() {
            return;
        }

        let tab_titles: Vec<Line> = self
            .tabs
            .iter()
            .map(|tab| {
                let style = if tab.has_unsaved_changes {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC)
                } else {
                    Style::default()
                };
                Line::from(Span::styled(tab.get_display_name(), style))
            })
            .collect();

        let tabs = RatatuiTabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow))
            .select(self.active_tab);

        f.render_widget(tabs, area);
    }

    pub fn render_close_confirmation(&self, f: &mut Frame, area: Rect) {
        if !self.show_close_confirmation {
            return;
        }

        let popup_area = centered_rect(50, 30, area);
        f.render_widget(Clear, popup_area);

        let tab_name = if let Some(index) = self.tab_to_close {
            self.tabs
                .get(index)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Close Tab with Unsaved Changes?",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("Tab: {}", tab_name)),
            Line::from(""),
            Line::from("This tab has unsaved changes that will be lost."),
            Line::from(""),
            Line::from("Y - Close anyway"),
            Line::from("N - Cancel"),
        ];

        let popup = Paragraph::new(text).block(
            Block::default()
                .title(" Confirm Close ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

        f.render_widget(popup, popup_area);
    }

    pub fn get_tabs_info(&self) -> String {
        if self.tabs.is_empty() {
            "No tabs open".to_string()
        } else {
            let unsaved_count = self.tabs.iter().filter(|t| t.has_unsaved_changes).count();
            if unsaved_count > 0 {
                format!("{} tabs ({} unsaved)", self.tabs.len(), unsaved_count)
            } else {
                format!("{} tabs", self.tabs.len())
            }
        }
    }
}

// Helper function for centering rectangles
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_tab_creation() {
        let tab = Tab::new(
            1,
            "test.rs".to_string(),
            PathBuf::from("test.rs"),
            "content".to_string(),
        );
        assert_eq!(tab.id, 1);
        assert_eq!(tab.name, "test.rs");
        assert!(!tab.has_unsaved_changes);
    }

    #[test]
    fn test_tab_manager_add_tab() {
        let mut manager = TabManager::new();
        let index = manager.add_tab(
            "test.rs".to_string(),
            PathBuf::from("test.rs"),
            "content".to_string(),
        );
        assert_eq!(index, 0);
        assert_eq!(manager.tab_count(), 1);
        assert_eq!(manager.get_active_tab_index(), 0);
    }

    #[test]
    fn test_tab_manager_close_tab() {
        let mut manager = TabManager::new();
        manager.add_tab(
            "test1.rs".to_string(),
            PathBuf::from("test1.rs"),
            "content1".to_string(),
        );
        manager.add_tab(
            "test2.rs".to_string(),
            PathBuf::from("test2.rs"),
            "content2".to_string(),
        );

        assert!(manager.close_tab(0).is_ok());
        assert_eq!(manager.tab_count(), 1);
        assert_eq!(manager.get_active_tab_index(), 0);
    }

    #[test]
    fn test_tab_manager_navigation() {
        let mut manager = TabManager::new();
        manager.add_tab(
            "test1.rs".to_string(),
            PathBuf::from("test1.rs"),
            "content1".to_string(),
        );
        manager.add_tab(
            "test2.rs".to_string(),
            PathBuf::from("test2.rs"),
            "content2".to_string(),
        );
        manager.add_tab(
            "test3.rs".to_string(),
            PathBuf::from("test3.rs"),
            "content3".to_string(),
        );

        assert_eq!(manager.get_active_tab_index(), 2);

        manager.next_tab();
        assert_eq!(manager.get_active_tab_index(), 0);

        manager.previous_tab();
        assert_eq!(manager.get_active_tab_index(), 2);
    }
}
