//! A linked list structure to store currently playing notes.

use slotmap::SlotMap;

pub struct Note<State> {
    /// The frequency of the note.
    pub freq: f32,
    /// The amplitude of the note.
    pub amp: f32,
    /// The time since the note started or was released, depending on `held`.
    pub time: f32,
    /// Whether the node is still being held.
    pub held: bool,
    /// The state of the oscillator.
    pub state: State,
}

pub enum NoteState {
    Holding(f32),
    Released(f32),
}

impl<St> Note<St> {
    pub fn held_state(&self, t_offset: f32) -> NoteState {
        if self.held {
            NoteState::Holding(self.time + t_offset)
        } else {
            NoteState::Released(self.time + t_offset)
        }
    }
}

slotmap::new_key_type! {
    pub struct NoteId;
}

struct ListEntry<St> {
    it: Note<St>,
    next: Option<NoteId>,
    prev: Option<NoteId>,
}

pub struct NoteList<St> {
    head: Option<NoteId>,
    tail: Option<NoteId>,
    entries: SlotMap<NoteId, ListEntry<St>>,
}

impl<St> NoteList<St> {
    pub fn new(cap: usize) -> Self {
        NoteList {
            head: None,
            tail: None,
            entries: SlotMap::with_capacity_and_key(cap),
        }
    }

    pub fn add(&mut self, note: Note<St>) -> NoteId {
        // Evict the oldest note if the list is full.
        if self.entries.len() == self.entries.capacity() {
            let key = self.head.unwrap();
            self.entries.remove(key);
            self.head = self.entries[key].next;
            self.entries[self.head.unwrap()].prev = None;
        }

        let key = self.entries.insert(ListEntry {
            it: note,
            next: None,
            prev: self.tail,
        });
        if let Some(tail) = self.tail {
            self.entries[tail].next = Some(key);
        }
        if self.head.is_none() {
            self.head = Some(key);
        }
        self.tail = Some(key);
        key
    }

    pub fn get_mut(&mut self, key: NoteId) -> Option<&mut Note<St>> {
        self.entries.get_mut(key).map(|entry| &mut entry.it)
    }

    pub fn remove(&mut self, key: NoteId) {
        let entry = self.entries.remove(key);
        if let Some(entry) = entry {
            if let Some(prev) = entry.prev {
                self.entries[prev].next = entry.next;
            } else {
                self.head = entry.next;
            }
            if let Some(next) = entry.next {
                self.entries[next].prev = entry.prev;
            } else {
                self.tail = entry.prev;
            }
        }
    }

    pub fn filter(&mut self, f: impl Fn(&Note<St>) -> bool) {
        let mut key = self.head;
        while let Some(k) = key {
            let next = self.entries[k].next;
            if !f(&self.entries[k].it) {
                self.remove(k);
            }
            key = next;
        }
    }

    pub fn notes_mut(&mut self) -> impl Iterator<Item = &mut Note<St>> {
        self.entries.values_mut().map(|entry| &mut entry.it)
    }
}
