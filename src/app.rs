use crossterm::event::{Event, KeyCode, KeyEventKind};
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use ratatui::widgets::ListState;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::data::{ApiData, Model, Provider};

#[derive(Debug, Clone)]
pub enum ListItem {
    Provider {
        id: String,
        name: String,
    },
    Model {
        provider_id: String,
        provider_name: String,
        model_id: String,
        model_name: String,
    },
}

#[derive(Debug, Clone)]
pub enum View {
    Level1,
    Level2 { provider_id: String },
}

struct FuzzyHit {
    item: ListItem,
    score: u32,
}

fn fuzzy_score(haystack: &str, query: &str, matcher: &mut Matcher) -> Option<u32> {
    if query.is_empty() {
        return Some(0);
    }
    let mut buf = Vec::new();
    let haystack_utf32 = Utf32Str::new(haystack, &mut buf);
    Pattern::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    )
    .score(haystack_utf32, matcher)
}

fn fuzzy_match_item(
    provider_id: &str,
    provider_name: &str,
    model_id: Option<(&str, &Model)>,
    query: &str,
    matcher: &mut Matcher,
) -> Option<FuzzyHit> {
    if query.is_empty() {
        return Some(FuzzyHit {
            item: if let Some((mid, model)) = model_id {
                ListItem::Model {
                    provider_id: provider_id.to_string(),
                    provider_name: provider_name.to_string(),
                    model_id: mid.to_string(),
                    model_name: model.name.clone(),
                }
            } else {
                ListItem::Provider {
                    id: provider_id.to_string(),
                    name: provider_name.to_string(),
                }
            },
            score: 0,
        });
    }

    if let Some((mid, model)) = model_id {
        // Match against combined haystacks: model fields individually, plus
        // combined "provider_name model_name" for cross-field fuzzy like "zencl"
        let combined = format!("{} {}", provider_name, model.name);
        let reversed = format!("{} {}", model.name, provider_name);
        let haystacks: Vec<&str> = vec![
            &combined,
            &reversed,
            model.name.as_str(),
            mid,
            model.family.as_deref().unwrap_or(""),
            provider_name,
        ];
        let mut best: Option<u32> = None;
        for hay in &haystacks {
            if hay.is_empty() {
                continue;
            }
            if let Some(s) = fuzzy_score(hay, query, matcher) {
                best = Some(best.map_or(s, |b| b.max(s)));
            }
        }

        best.map(|score| FuzzyHit {
            item: ListItem::Model {
                provider_id: provider_id.to_string(),
                provider_name: provider_name.to_string(),
                model_id: mid.to_string(),
                model_name: model.name.clone(),
            },
            score,
        })
    } else {
        // Provider-only match
        let provider_score = fuzzy_score(provider_name, query, matcher).max(fuzzy_score(
            provider_id,
            query,
            matcher,
        ));
        provider_score.map(|score| FuzzyHit {
            item: ListItem::Provider {
                id: provider_id.to_string(),
                name: provider_name.to_string(),
            },
            score,
        })
    }
}

pub struct App {
    pub data: ApiData,
    pub view: View,
    pub level1_input: Input,
    pub level2_input: Input,
    pub list_state: ListState,
    pub filtered_items: Vec<ListItem>,
    pub should_quit: bool,
    pub detail_scroll: u16,
    pub detail_height: u16,
    pub detail_content_height: u16,
}

impl App {
    pub fn new(data: ApiData) -> Self {
        let mut app = Self {
            data,
            view: View::Level1,
            level1_input: Input::default(),
            level2_input: Input::default(),
            list_state: ListState::default(),
            filtered_items: Vec::new(),
            should_quit: false,
            detail_scroll: 0,
            detail_height: 0,
            detail_content_height: 0,
        };
        app.update_filtered();
        if !app.filtered_items.is_empty() {
            app.list_state.select(Some(0));
        }
        app
    }

    pub fn current_input(&self) -> &Input {
        match self.view {
            View::Level1 => &self.level1_input,
            View::Level2 { .. } => &self.level2_input,
        }
    }

    pub fn current_input_mut(&mut self) -> &mut Input {
        match self.view {
            View::Level1 => &mut self.level1_input,
            View::Level2 { .. } => &mut self.level2_input,
        }
    }

    pub fn update_filtered(&mut self) {
        let query = self.current_input().value().to_lowercase();
        self.filtered_items = match &self.view {
            View::Level1 => self.filter_level1(&query),
            View::Level2 { provider_id } => self.filter_level2(provider_id, &query),
        };
        let selected = self.list_state.selected().unwrap_or(0);
        if self.filtered_items.is_empty() {
            self.list_state.select(None);
        } else if selected >= self.filtered_items.len() {
            self.list_state.select(Some(self.filtered_items.len() - 1));
        }
    }

    fn filter_level1(&self, query: &str) -> Vec<ListItem> {
        let mut matcher = Matcher::new(Config::DEFAULT);

        let mut provider_ids: Vec<_> = self.data.keys().collect();
        provider_ids.sort_by(|a, b| {
            self.data[*a]
                .name
                .to_lowercase()
                .cmp(&self.data[*b].name.to_lowercase())
        });

        let mut providers: Vec<FuzzyHit> = Vec::new();
        let mut models: Vec<FuzzyHit> = Vec::new();

        for pid in provider_ids {
            let provider = &self.data[pid];

            // Try provider match
            if let Some(hit) = fuzzy_match_item(pid, &provider.name, None, query, &mut matcher) {
                providers.push(hit);
            }

            // Try each model
            if !query.is_empty() {
                for (mid, model) in &provider.models {
                    if let Some(hit) = fuzzy_match_item(
                        pid,
                        &provider.name,
                        Some((mid.as_str(), model)),
                        query,
                        &mut matcher,
                    ) {
                        models.push(hit);
                    }
                }
            }
        }

        // Sort each group by score descending
        providers.sort_by(|a, b| b.score.cmp(&a.score));
        models.sort_by(|a, b| b.score.cmp(&a.score));

        // Providers first, then models
        providers
            .into_iter()
            .chain(models)
            .map(|h| h.item)
            .collect()
    }

    fn filter_level2(&self, provider_id: &str, query: &str) -> Vec<ListItem> {
        let provider = match self.data.get(provider_id) {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut matcher = Matcher::new(Config::DEFAULT);
        let mut hits: Vec<FuzzyHit> = Vec::new();

        for (mid, model) in &provider.models {
            if let Some(hit) = fuzzy_match_item(
                provider_id,
                &provider.name,
                Some((mid.as_str(), model)),
                query,
                &mut matcher,
            ) {
                hits.push(hit);
            }
        }

        hits.sort_by(|a, b| b.score.cmp(&a.score));
        hits.into_iter().map(|h| h.item).collect()
    }

    pub fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return;
            }

            match key.code {
                KeyCode::Char('c')
                    if key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    self.should_quit = true;
                }
                KeyCode::Esc => match self.view {
                    View::Level2 { .. } => {
                        self.view = View::Level1;
                        self.detail_scroll = 0;
                        self.update_filtered();
                        if !self.filtered_items.is_empty() {
                            self.list_state.select(Some(0));
                        }
                    }
                    View::Level1 => {
                        self.level1_input.reset();
                        self.detail_scroll = 0;
                        self.update_filtered();
                        if !self.filtered_items.is_empty() {
                            self.list_state.select(Some(0));
                        }
                    }
                },
                KeyCode::Up => {
                    self.move_up();
                    self.detail_scroll = 0;
                }
                KeyCode::Down => {
                    self.move_down();
                    self.detail_scroll = 0;
                }
                KeyCode::Enter => {
                    self.handle_enter();
                }
                KeyCode::PageUp => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(5);
                    self.clamp_detail_scroll();
                }
                KeyCode::PageDown => {
                    self.detail_scroll = self.detail_scroll.saturating_add(5);
                    self.clamp_detail_scroll();
                }
                _ => {
                    let prev = self.current_input().value().to_string();
                    self.current_input_mut().handle_event(event);
                    if self.current_input().value() != prev {
                        self.detail_scroll = 0;
                        self.update_filtered();
                        if !self.filtered_items.is_empty() {
                            self.list_state.select(Some(0));
                        }
                    }
                }
            }
        }
    }

    fn move_up(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn move_down(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len() - 1 {
                    self.filtered_items.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn clamp_detail_scroll(&mut self) {
        let max_scroll = self
            .detail_content_height
            .saturating_sub(self.detail_height);
        if self.detail_scroll > max_scroll {
            self.detail_scroll = max_scroll;
        }
    }

    fn handle_enter(&mut self) {
        let idx = match self.list_state.selected() {
            Some(i) => i,
            None => return,
        };
        if idx >= self.filtered_items.len() {
            return;
        }
        if let ListItem::Provider { id, .. } = &self.filtered_items[idx] {
            match &self.view {
                View::Level1 => {
                    self.view = View::Level2 {
                        provider_id: id.clone(),
                    };
                    self.level2_input = Input::default();
                    self.detail_scroll = 0;
                    self.update_filtered();
                    if !self.filtered_items.is_empty() {
                        self.list_state.select(Some(0));
                    }
                }
                View::Level2 { .. } => {}
            }
        }
    }

    pub fn get_selected(&self) -> Option<&ListItem> {
        let idx = self.list_state.selected()?;
        self.filtered_items.get(idx)
    }

    pub fn get_provider(&self, id: &str) -> Option<&Provider> {
        self.data.get(id)
    }

    pub fn get_model(&self, provider_id: &str, model_id: &str) -> Option<&Model> {
        self.data.get(provider_id)?.models.get(model_id)
    }
}
