use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;

pub struct Sampler<'a> {
    choices: &'a [f32],
    dist: WeightedIndex<f32>,
}

impl<'a> Sampler<'a> {
    pub fn linear(choices: &'a [f32], start: f32, end: f32) -> Self {
        assert!(end > start);
        assert!(choices.len() > 1);
        assert!(start.is_sign_positive() && end.is_sign_positive());

        let len = choices.len();
        let step = (end - start) / (len - 1) as f32;
        let dist = WeightedIndex::new((0..len).map(|i| step * i as f32)).unwrap();

        Self { choices, dist }
    }

    pub fn sample(&self, rng: &mut impl Rng) -> f32 {
        self.choices[self.dist.sample(rng)]
    }

    pub fn iter(&self, rng: &mut impl Rng, samples: usize) -> impl Iterator<Item = f32> {
        (0..samples).map(|_| self.sample(rng))
    }
}
