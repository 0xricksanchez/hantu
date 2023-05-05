use grammar_mutator::TokenIdentifier;
use prng::{Generator, Rng};

pub type GenerateFn = Box<dyn Fn(usize, TokenIdentifier, &mut Rng<Generator>, &mut Vec<u8>)>;

pub struct GrammarCaller {
    pub generate_fn: GenerateFn,
}

impl GrammarCaller {
    pub fn call_generate(
        &self,
        depth: usize,
        id: TokenIdentifier,
        prng: &mut Rng<Generator>,
        out: &mut Vec<u8>,
    ) {
        (self.generate_fn)(depth, id, prng, out);
    }
}

#[allow(clippy::ptr_arg)]
fn dummy_generate(
    _depth: usize,
    _id: TokenIdentifier,
    _prng: &mut Rng<Generator>,
    _out: &mut Vec<u8>,
) {
    // This function does nothing.
}

impl Default for GrammarCaller {
    fn default() -> Self {
        Self {
            generate_fn: Box::new(dummy_generate),
        }
    }
}
