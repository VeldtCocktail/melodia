// melodia/src/queue.rs
// Playback queue — ordered list of track IDs with reordering support

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Queue {
    pub items: Vec<QueueItem>,
    pub current_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub track_id: String,
    /// Human-readable label for display
    pub display_title: String,
    pub display_artist: String,
    pub duration_str: String,
}

impl Queue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace entire queue with a new list (e.g., play all from library).
    pub fn set(&mut self, items: Vec<QueueItem>, start_index: usize) {
        self.items = items;
        self.current_index = if self.items.is_empty() { None } else { Some(start_index.min(self.items.len() - 1)) };
    }

    /// Append a single track to the end of the queue.
    pub fn enqueue(&mut self, item: QueueItem) {
        self.items.push(item);
        if self.current_index.is_none() {
            self.current_index = Some(0);
        }
    }

    /// Insert a track right after the current position ("play next").
    pub fn enqueue_next(&mut self, item: QueueItem) {
        let insert_pos = match self.current_index {
            Some(i) => i + 1,
            None => 0,
        };
        self.items.insert(insert_pos, item);
        if self.current_index.is_none() {
            self.current_index = Some(0);
        }
    }

    pub fn current_track_id(&self) -> Option<&str> {
        self.current_index
            .and_then(|i| self.items.get(i))
            .map(|item| item.track_id.as_str())
    }

    pub fn current_item(&self) -> Option<&QueueItem> {
        self.current_index.and_then(|i| self.items.get(i))
    }

    pub fn has_next(&self) -> bool {
        match self.current_index {
            Some(i) => i + 1 < self.items.len(),
            None => false,
        }
    }

    pub fn has_prev(&self) -> bool {
        match self.current_index {
            Some(i) => i > 0,
            None => false,
        }
    }

    pub fn advance(&mut self) -> Option<&str> {
        if let Some(i) = self.current_index {
            if i + 1 < self.items.len() {
                self.current_index = Some(i + 1);
                return self.current_track_id();
            } else {
                self.current_index = None;
            }
        }
        None
    }

    pub fn go_prev(&mut self) -> Option<&str> {
        if let Some(i) = self.current_index {
            if i > 0 {
                self.current_index = Some(i - 1);
                return self.current_track_id();
            }
        }
        self.current_track_id()
    }

    pub fn remove(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        self.items.remove(index);
        if self.items.is_empty() {
            self.current_index = None;
        } else if let Some(ci) = self.current_index {
            if index < ci {
                self.current_index = Some(ci - 1);
            } else if index == ci {
                // Playing item removed — stay at same index (now next track), or clamp
                let new_idx = ci.min(self.items.len() - 1);
                self.current_index = Some(new_idx);
            }
            // index > ci — no change needed
        }
    }

    /// Move item at `from` to `to` (drag-and-drop reorder).
    pub fn move_item(&mut self, from: usize, to: usize) {
        if from == to || from >= self.items.len() || to >= self.items.len() {
            return;
        }
        let item = self.items.remove(from);
        self.items.insert(to, item);

        // Update current_index to follow the moved item
        if let Some(ci) = self.current_index {
            if ci == from {
                self.current_index = Some(to);
            } else if from < ci && to >= ci {
                self.current_index = Some(ci - 1);
            } else if from > ci && to <= ci {
                self.current_index = Some(ci + 1);
            }
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.current_index = None;
    }

    pub fn jump_to(&mut self, index: usize) {
        if index < self.items.len() {
            self.current_index = Some(index);
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
