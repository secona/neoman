use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use reqwest::Url;
use std::error;
use strum::IntoEnumIterator;

use tui_menu::{MenuItem, MenuState};
use tui_tree_widget::TreeItem;

use crate::{
    component::{
        requestbar::{RequestBar, RequestMenu},
        responsebar::ResponseBar,
        sidebar::SideBar,
        tabbar::TabBar,
        urlbar::{InputMode, Method, UrlBar},
    },
    items::{Item, StatefulTree},
};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub selected: Selected,
    pub sidebar: SideBar,
    pub settings: Settings,
    pub tabs: TabBar,
    pub urlbar: UrlBar,
    pub requestbar: RequestBar,
    pub responsebar: ResponseBar,
}

#[derive(Debug, Default, strum::Display, strum::EnumIter, PartialEq)]
pub enum Selected {
    #[default]
    Sidebar,
    Tabs,
    MethodBar,
    Urlbar,
    RequestTab,
    Requestbar,
    Responsebar,
}

#[derive(Debug)]
pub struct Settings {
    pub show_sidebar: bool,
    pub show_help: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_sidebar: true,
            show_help: false,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let mut tree = StatefulTree::with_items(vec![
            TreeItem::new_leaf(Item::new("a")),
            TreeItem::new(
                Item::new("b"),
                vec![
                    TreeItem::new_leaf(Item::new("c")),
                    TreeItem::new(
                        Item::new("d"),
                        vec![
                            TreeItem::new_leaf(Item::new("e")),
                            TreeItem::new_leaf(Item::new("f")),
                        ],
                    ),
                    TreeItem::new_leaf(Item::new("g")),
                ],
            ),
            TreeItem::new_leaf(Item::new("d")),
        ]);
        tree.first();

        let tabs = tree.items.iter().map(|i| i.inner().clone()).collect();

        Self {
            running: true,
            selected: Selected::Urlbar,
            sidebar: SideBar {
                size: 25,
                selected: 0,
                tree,
            },
            settings: Settings {
                show_sidebar: true,
                show_help: false,
            },
            tabs: TabBar { selected: 0, tabs },
            urlbar: UrlBar {
                title: String::from("https://api.kanye.rest/?q=a"),
                text: String::from("https://api.kanye.rest/?q=a"),
                cursor_position: 0,
                input_mode: InputMode::Normal,
                method: Method::Get,
                method_menu: MenuState::new(vec![MenuItem::group(
                    Method::default().to_string(),
                    Method::iter()
                        .map(|m| MenuItem::item(m.to_string(), m))
                        .collect(),
                )]),
            },
            requestbar: RequestBar {
                body: String::new(),
                request_menu: RequestMenu::Params,
            },
            responsebar: ResponseBar {
                body: String::new(),
            },
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn toggle_sidebar(&mut self) {
        if self.selected == Selected::Sidebar {
            self.selected = Selected::Tabs;
        }

        self.settings.show_sidebar = !self.settings.show_sidebar;
    }

    pub fn sidebar_size(&self) -> u16 {
        match self.settings.show_sidebar {
            true => self.sidebar.size,
            false => 0,
        }
    }

    pub async fn request(&mut self) {
        let client = reqwest::Client::new();

        let method = match self.urlbar.method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Patch => reqwest::Method::PATCH,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
        };

        let url = match Url::parse(&self.urlbar.text) {
            Ok(url) => url,
            Err(_) => Url::parse(&format!("https://{}", &self.urlbar.text)).unwrap(),
        };

        let mut req = reqwest::Request::new(method, url);
        req.body_mut().replace(self.requestbar.body.clone().into());

        let res = client.execute(req).await.unwrap();
        let body = res.text().await.unwrap_or_default();

        self.responsebar.body = body;
    }

    pub async fn handle_key_events(&mut self, key_event: KeyEvent) -> AppResult<()> {
        // global key handlers
        match key_event.code {
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.quit();
                }
            }

            KeyCode::Char('b') | KeyCode::Char('B') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.toggle_sidebar();
                }
            }

            KeyCode::Esc | KeyCode::Char('q') => {
                // if !self.urlbar.method_menu.is_open() {
                //     self.quit();
                // }
            }

            KeyCode::Tab | KeyCode::Char('.') | KeyCode::Char(']') => {
                self.selected = match self.selected {
                    Selected::Sidebar => Selected::Tabs,
                    Selected::Tabs => Selected::MethodBar,
                    Selected::MethodBar => Selected::Urlbar,
                    Selected::Urlbar => Selected::RequestTab,
                    Selected::RequestTab => Selected::Requestbar,
                    Selected::Requestbar => Selected::Responsebar,
                    Selected::Responsebar => Selected::Sidebar,
                };
            }

            KeyCode::Char(',') | KeyCode::Char('[') => {
                self.selected = match self.selected {
                    Selected::Sidebar => Selected::Responsebar,
                    Selected::Tabs => Selected::Sidebar,
                    Selected::MethodBar => Selected::Tabs,
                    Selected::Urlbar => Selected::MethodBar,
                    Selected::RequestTab => Selected::Urlbar,
                    Selected::Requestbar => Selected::RequestTab,
                    Selected::Responsebar => Selected::Requestbar,
                }
            }

            // Other handlers you could add here.
            _ => {}
        }

        match self.selected {
            Selected::Sidebar => match key_event.code {
                KeyCode::Char(' ') | KeyCode::Char('o') | KeyCode::Enter => {
                    if let Some(item) = self.sidebar.selected() {
                        if item.children().is_empty() {
                            let item_name = item.inner().to_string();
                            match self
                                .tabs
                                .tabs
                                .iter()
                                .enumerate()
                                .find(|(_, item)| item.borrow().name == item_name)
                                .map(|(i, _)| i)
                            {
                                Some(i) => {
                                    self.tabs.selected = i;
                                    self.selected = Selected::Tabs;
                                }
                                None => {
                                    self.tabs.add(item.inner().clone());
                                    self.tabs.selected = self.tabs.tabs.len() - 1;
                                    self.selected = Selected::Tabs;
                                }
                            }
                        } else {
                            self.sidebar.tree.toggle();
                        }
                    }
                }

                KeyCode::Left => self.sidebar.tree.left(),
                KeyCode::Right => self.sidebar.tree.right(),
                KeyCode::Down => self.sidebar.tree.down(),
                KeyCode::Up => self.sidebar.tree.up(),
                KeyCode::Home => self.sidebar.tree.first(),
                KeyCode::End => self.sidebar.tree.last(),

                _ => {}
            },
            Selected::Tabs => match key_event.code {
                KeyCode::Left => {
                    self.tabs.left();
                }
                KeyCode::Right => {
                    self.tabs.right();
                }
                KeyCode::Down => {
                    self.tabs.right();
                }
                KeyCode::Up => {
                    self.tabs.left();
                }
                KeyCode::Home => {
                    self.tabs.first();
                }
                KeyCode::End => {
                    self.tabs.last();
                }
                KeyCode::Enter => {}
                _ => {}
            },
            Selected::MethodBar => {
                match key_event.code {
                    KeyCode::Char('h') | KeyCode::Left => self.urlbar.method_menu.left(),
                    KeyCode::Char('l') | KeyCode::Right => self.urlbar.method_menu.right(),
                    KeyCode::Char('j') | KeyCode::Down => self.urlbar.method_menu.down(),
                    KeyCode::Char('k') | KeyCode::Up => self.urlbar.method_menu.up(),
                    KeyCode::Esc => self.urlbar.method_menu.reset(),
                    KeyCode::Enter => self.urlbar.method_menu.select(),
                    _ => {}
                };

                for e in self.urlbar.method_menu.drain_events() {
                    match e {
                        tui_menu::MenuEvent::Selected(item) => {
                            self.urlbar.method_menu.set_child_name(0, item.to_string());
                            self.urlbar.method_menu.close();
                            self.urlbar.method = item;
                        }
                    }
                }
            }
            Selected::Urlbar => match self.urlbar.input_mode {
                InputMode::Normal => match key_event.code {
                    KeyCode::Enter | KeyCode::Char('i') => self.urlbar.input_mode = InputMode::Insert,
                    KeyCode::Char('o') => {
                        self.request().await;
                    }
                    _ => {}
                },
                InputMode::Insert => {
                    match key_event.code {
                        // KeyCode::Esc => self.urlbar.input_mode = InputMode::Normal,
                        KeyCode::Enter => self.urlbar.input_mode = InputMode::Normal,
                        KeyCode::Char(c) => {
                            self.urlbar.text.insert(self.urlbar.cursor_position, c);
                            self.urlbar.cursor_position += 1;
                        }
                        KeyCode::Backspace => {
                            if self.urlbar.cursor_position > 0 {
                                self.urlbar.cursor_position -= 1;
                                self.urlbar.text.remove(self.urlbar.cursor_position);
                            }
                        }
                        KeyCode::Delete => {
                            if self.urlbar.cursor_position < self.urlbar.text.len() {
                                self.urlbar.text.remove(self.urlbar.cursor_position);
                            }
                        }
                        KeyCode::Left => {
                            if self.urlbar.cursor_position > 0 {
                                self.urlbar.cursor_position -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if self.urlbar.cursor_position < self.urlbar.text.len() {
                                self.urlbar.cursor_position += 1;
                            }
                        }
                        KeyCode::Home => {
                            self.urlbar.cursor_position = 0;
                        }
                        KeyCode::End => {
                            self.urlbar.cursor_position = self.urlbar.text.len();
                        }
                        _ => {}
                    }
                }
            },
            Selected::RequestTab => {
                match key_event.code {
                    KeyCode::Char('h') | KeyCode::Left => self.requestbar.left(),
                    KeyCode::Char('l') | KeyCode::Right => self.requestbar.right(),
                    KeyCode::Char('j') | KeyCode::Down => self.requestbar.left(),
                    KeyCode::Char('k') | KeyCode::Up => self.requestbar.right(),

                    _ => {}
                };
            }
            Selected::Requestbar => {}
            Selected::Responsebar => {}
        }
        Ok(())
    }
}
