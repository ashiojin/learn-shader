use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub struct RandomPlugin;

#[derive(Resource, Debug)]
pub struct RandomSource(ChaCha8Rng);

impl RandomSource {
    pub fn rnd_mut(&mut self) -> &mut ChaCha8Rng {
        &mut self.0
    }
}

fn setup_random_source(mut commands: Commands) {
    let rng = ChaCha8Rng::seed_from_u64(42); // You can choose any seed you like
    commands.insert_resource(RandomSource(rng));
}

impl Plugin for RandomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_random_source);
    }
}
