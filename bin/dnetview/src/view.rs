use crate::model::NodeInfo;
use std::collections::{HashMap, HashSet};
use tui::widgets::ListState;

#[derive(Clone)]
pub struct View {
    pub id_list: IdListView,
    pub info_list: InfoListView,
}

impl View {
    pub fn new(id_list: IdListView, info_list: InfoListView) -> View {
        View { id_list, info_list }
    }

    pub fn update(&mut self, infos: HashMap<String, NodeInfo>) {
        for (id, info) in infos.clone() {
            self.id_list.node_id.insert(id.clone());
            self.info_list.infos.insert(id, info);
        }
    }
}

#[derive(Clone)]
pub struct IdListView {
    pub state: ListState,
    pub node_id: HashSet<String>,
}

impl IdListView {
    pub fn new(node_id: HashSet<String>) -> IdListView {
        IdListView { state: ListState::default(), node_id }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.node_id.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.node_id.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[derive(Clone)]
pub struct InfoListView {
    pub index: usize,
    pub infos: HashMap<String, NodeInfo>,
}

impl InfoListView {
    pub fn new(infos: HashMap<String, NodeInfo>) -> InfoListView {
        let index = 0;

        InfoListView { index, infos }
    }

    pub async fn next(&mut self) {
        self.index = (self.index + 1) % self.infos.len();
    }

    pub async fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.infos.len() - 1;
        }
    }
}
