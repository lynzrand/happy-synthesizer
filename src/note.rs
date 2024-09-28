//! A linked list structure to store currently playing notes.

use slotmap::SlotMap;

use crate::AdsrEnvelope;

pub struct Note {
    /// The frequency of the note.
    pub freq: f32,
    /// The amplitude of the note.
    pub amp: f32,
    /// The time since the note started or was released, depending on `held`.
    pub time: f32,
    /// Whether the node is still being held.
    pub held: bool,
}

slotmap::new_key_type! {
    pub struct NoteId;
}

struct ListEntry {
    it: Note,
    next: Option<NoteId>,
    prev: Option<NoteId>,
}

pub struct NoteList {
    head: Option<NoteId>,
    tail: Option<NoteId>,
    entries: SlotMap<NoteId, ListEntry>,
}

impl NoteList {
    pub fn new(cap: usize) -> Self {
        NoteList {
            head: None,
            tail: None,
            entries: SlotMap::with_capacity_and_key(cap),
        }
    }

    pub fn add(&mut self, note: Note) -> NoteId {
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

    pub fn get(&self, key: NoteId) -> Option<&Note> {
        self.entries.get(key).map(|entry| &entry.it)
    }

    pub fn get_mut(&mut self, key: NoteId) -> Option<&mut Note> {
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

    pub fn filter(&mut self, f: impl Fn(&Note) -> bool) {
        let mut key = self.head;
        while let Some(k) = key {
            let next = self.entries[k].next;
            if !f(&self.entries[k].it) {
                self.remove(k);
            }
            key = next;
        }
    }

    pub fn release(&mut self, key: NoteId) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.it.held = false;
            entry.it.time = 0.0;
        }
    }

    pub fn notes_mut(&mut self) -> impl Iterator<Item = &mut Note> {
        self.entries.values_mut().map(|entry| &mut entry.it)
    }
}