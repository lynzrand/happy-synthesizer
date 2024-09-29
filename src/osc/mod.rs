pub mod harmonic;
pub mod noise;
pub mod saw;
pub mod sine;
pub mod square;
pub mod triangle;

pub trait Oscillator {
    /// This type should store the state of the oscillator.
    type State;

    /// Create a new state for the oscillator.
    fn create_state(&self) -> Self::State;

    /// Fill the buffer with samples of the oscillator.
    ///
    /// The oscillator implementation should **add** its samples to the buffer, instead of
    /// overwriting them, in order to allow oscillators to be composable.
    ///
    /// - `delta_t` is the time between samples, in seconds.
    /// - `freq` is the base frequency of the oscillator, in Hz.
    /// - `amp` is the amplitude of the oscillator.
    fn fill_samples(
        &self,
        state: &mut Self::State,
        buffer: &mut [f32],
        delta_t: f32,
        freq: f32,
        amp: f32,
    );
}
