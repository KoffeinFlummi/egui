use std::collections::{HashMap, HashSet};

use crate::{
    containers::{collapsing_header, floating, resize, scroll_area},
    Id, Layer, Pos2, Rect,
};

#[derive(Clone, Debug, Default, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(default)]
pub struct Memory {
    /// The widget being interacted with (e.g. dragged, in case of a slider).
    #[serde(skip)]
    pub(crate) active_id: Option<Id>,

    /// The widget with keyboard focus (i.e. a text input field).
    #[serde(skip)]
    pub(crate) kb_focus_id: Option<Id>,

    // states of various types of widgets
    pub(crate) collapsing_headers: HashMap<Id, collapsing_header::State>,
    pub(crate) scroll_areas: HashMap<Id, scroll_area::State>,
    pub(crate) resize: HashMap<Id, resize::State>,

    floating: HashMap<Id, floating::State>,
    /// Top is last
    floating_order: Vec<Id>,
    floating_visible_last_frame: HashSet<Id>,
    floating_visible_current_frame: HashSet<Id>,
}

impl Memory {
    pub(crate) fn get_floating(&mut self, id: Id) -> Option<floating::State> {
        self.floating.get(&id).cloned()
    }

    pub(crate) fn floating_order(&self) -> &[Id] {
        &self.floating_order
    }

    pub(crate) fn set_floating_state(&mut self, id: Id, state: floating::State) {
        self.floating_visible_current_frame.insert(id);
        let did_insert = self.floating.insert(id, state).is_none();
        if did_insert {
            self.floating_order.push(id);
        }
    }

    /// TODO: call once at the start of the frame for the current mouse pos
    pub fn layer_at(&self, pos: Pos2) -> Layer {
        for floating_id in self.floating_order.iter().rev() {
            if self.floating_visible_last_frame.contains(floating_id)
                || self.floating_visible_current_frame.contains(floating_id)
            {
                if let Some(state) = self.floating.get(floating_id) {
                    let rect = Rect::from_min_size(state.pos, state.size);
                    if rect.contains(pos) {
                        return Layer::Window(*floating_id);
                    }
                }
            }
        }
        Layer::Background
    }

    pub fn move_floating_to_top(&mut self, id: Id) {
        if self.floating_order.last() == Some(&id) {
            return; // common case early-out
        }
        if let Some(index) = self.floating_order.iter().position(|x| *x == id) {
            self.floating_order.remove(index);
        }
        self.floating_order.push(id);
        self.floating_visible_current_frame.insert(id);
    }

    pub(crate) fn begin_frame(&mut self) {
        self.floating_visible_last_frame = std::mem::take(&mut self.floating_visible_current_frame);
    }
}
