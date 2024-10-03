use crate::note::NoteState;

pub trait Envelope {
    /// Sample the envelope at a given time.
    fn sample(&self, state: NoteState) -> f32;

    /// Returns whether the note will not make any sound anymore.
    fn note_ended(&self, state: NoteState) -> bool;
}

pub mod adsr;
