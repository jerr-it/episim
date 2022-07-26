use quadtree::Positioned;
use rand::{prelude::StdRng, Rng, SeedableRng};
use vector::Vector2;

use crate::CONFIG;

#[derive(PartialEq)]
pub enum InfectionStatus {
    Susceptible,
    Infected(u32), // time the entity will remain infected. The entity will either recover or die.
    Recovered(u32), // time of days the entity will remain recovered (cannot be infected again). The entity will be susceptible again if this counter reaches 0.
    Dead,
}

pub struct Entity {
    position: Vector2<f32>, // Used for calculating entity movement.
    velocity: Vector2<f32>,
    acceleration: Vector2<f32>,

    health: InfectionStatus,

    hospitalized: bool,
    mobile: bool, // True if the entity can move (Neither dead, nor in Hospital). False if it is immobile.

    age: u8,

    rng: StdRng,
}

// Required for the quadtree to work.
impl Positioned for Entity {
    fn position(&self) -> &Vector2<f32> {
        &self.position
    }
}

impl Entity {
    pub fn new() -> Entity {
        let mut rng: StdRng = rand::rngs::StdRng::from_entropy();

        let x_position = rng.gen_range(0.0..CONFIG.core.dimensions.0 as f32);
        let y_position = rng.gen_range(0.0..CONFIG.core.dimensions.1 as f32);

        let chance = rng.gen::<f32>();
        let infected = chance < CONFIG.core.initial_infected;

        let chance = rng.gen::<f32>();
        let mobile = chance < CONFIG.core.initial_mobile;

        let age = CONFIG.sample_age(&mut rng);

        let speed_range = (-CONFIG.core.max_velocity * 0.1)..(CONFIG.core.max_velocity * 0.1);

        Entity {
            position: Vector2::new(x_position, y_position),
            velocity: Vector2::new(
                rng.gen_range(speed_range.clone()),
                rng.gen_range(speed_range),
            ),
            acceleration: Vector2::new(0.0, 0.0),
            health: if infected {
                InfectionStatus::Infected(CONFIG.core.infected_period)
            } else {
                InfectionStatus::Susceptible
            },
            hospitalized: false,
            mobile,
            age,
            rng,
        }
    }

    /// Simple model for force based movement.
    /// Maximum velocity is limited to CONFIG.core.max_velocity.
    pub fn update_movement(&mut self) {
        if !self.mobile {
            return;
        }

        self.velocity.clamp_mag(CONFIG.core.max_velocity);
        self.position += self.velocity;
        self.velocity += self.acceleration;
        self.acceleration *= 0.0;

        self.check_boundaries();
    }

    /// Check if the entity is outside the boundaries.
    /// Reverse the velocity if it is.
    fn check_boundaries(&mut self) {
        if self.position.x < 0.0 {
            self.velocity.x *= -1.0;
        } else if self.position.x >= CONFIG.core.dimensions.0 as f32 {
            self.velocity.x *= -1.0;
        }

        if self.position.y < 0.0 {
            self.velocity.y *= -1.0;
        } else if self.position.y >= CONFIG.core.dimensions.1 as f32 {
            self.velocity.y *= -1.0;
        }
    }

    /// Run an infection test on this entity.
    pub fn test(&mut self) -> bool {
        match self.health {
            InfectionStatus::Susceptible => {
                let rng = self.rand();
                !(rng < CONFIG.core.test_true_negative)
            }
            InfectionStatus::Infected(_) => {
                let rng = self.rand();
                rng < CONFIG.core.test_true_positive
            }
            InfectionStatus::Recovered(_) => {
                let rng = self.rand();
                !(rng < CONFIG.core.test_true_negative)
            }
            InfectionStatus::Dead => false,
        }
    }

    pub fn apply_force(&mut self, force: Vector2<f32>) {
        self.acceleration += force;
    }

    pub fn susceptible(&mut self) {
        self.health = InfectionStatus::Susceptible;
        self.mobile = true;
    }

    pub fn recover(&mut self) {
        self.health = InfectionStatus::Recovered(CONFIG.core.recovered_period);
    }

    pub fn die(&mut self) {
        self.health = InfectionStatus::Dead;
    }

    pub fn is_hospitalized(&self) -> bool {
        self.hospitalized
    }

    pub fn set_hospitalized(&mut self) {
        self.hospitalized = true;
        self.mobile = false;
    }

    pub fn unset_hospitalized(&mut self) {
        self.hospitalized = false;
        self.mobile = true;
    }

    pub fn infect(&mut self) {
        self.health = InfectionStatus::Infected(CONFIG.core.infected_period);
    }

    pub fn status(&self) -> &InfectionStatus {
        &self.health
    }

    pub fn is_dead(&self) -> bool {
        self.health == InfectionStatus::Dead
    }

    pub fn age(&self) -> u8 {
        self.age
    }

    pub fn health(&self) -> &InfectionStatus {
        &self.health
    }

    pub fn rand(&mut self) -> f32 {
        self.rng.gen::<f32>()
    }

    /// Performs the transition between
    /// the existing epidemic model groups.
    pub fn update_status(&mut self) {
        match self.health {
            InfectionStatus::Infected(time_remaining) => {
                if time_remaining <= 0 {
                    let chance = self.rand();

                    if chance <= (CONFIG.survival_chance)(self) {
                        self.recover();
                    } else {
                        self.die();
                    }
                } else {
                    self.health = InfectionStatus::Infected(time_remaining - 1);
                }
            }
            InfectionStatus::Recovered(time_remaining) => {
                if time_remaining <= 0 {
                    self.susceptible();
                } else {
                    self.health = InfectionStatus::Recovered(time_remaining - 1);
                }
            }
            _ => {}
        }
    }
}
