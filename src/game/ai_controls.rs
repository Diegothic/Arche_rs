use bevy::prelude::*;
use rand::Rng;

use super::GameState;

pub struct AIControlsPlugin;

impl Plugin for AIControlsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AIControls::new());
    }
}

#[derive(Resource)]
pub struct AIControls {
    net: NeuralNetwork,
    pull_power: f32,
    pull_angle: f32,
}

impl AIControls {
    fn new() -> Self {
        Self {
            net: NeuralNetwork::new(),
            pull_power: 0.0,
            pull_angle: 0.0,
        }
    }

    pub fn think(&mut self, game_state: &GameState) {
        let input = vec![game_state.enemy_height, game_state.player_height];
        let output = self.net.calculate_output(input);
        println!("{:?}", output);
        self.pull_power = output[0];
        self.pull_angle = output[1];
    }

    pub fn get_pull_power(&self) -> f32 {
        self.pull_power
    }

    pub fn get_pull_angle(&self) -> f32 {
        self.pull_angle
    }
}

struct NeuralNetwork {
    weights: Vec<Vec<Vec<f32>>>,
    biases: Vec<Vec<f32>>,
}

impl NeuralNetwork {
    fn new() -> Self {
        let mut random = rand::thread_rng();
        let weights = vec![
            vec![
                vec![random.gen_range(-1.0..=1.0), random.gen_range(-1.0..=1.0)],
                vec![random.gen_range(-1.0..=1.0), random.gen_range(-1.0..=1.0)],
                vec![random.gen_range(-1.0..=1.0), random.gen_range(-1.0..=1.0)],
            ],
            vec![
                vec![random.gen_range(-1.0..=1.0), random.gen_range(-1.0..=1.0)],
                vec![random.gen_range(-1.0..=1.0), random.gen_range(-1.0..=1.0)],
            ],
        ];

        let biases = vec![
            vec![
                random.gen_range(-1.0..=1.0),
                random.gen_range(-1.0..=1.0),
                random.gen_range(-1.0..=1.0),
            ],
            vec![random.gen_range(-1.0..=1.0), random.gen_range(-1.0..=1.0)],
        ];

        Self { weights, biases }
    }

    fn get_weight(&self, layer: usize, neuron: usize, input: usize) -> f32 {
        self.weights[layer][neuron][input]
    }

    fn get_bias(&self, layer: usize, neuron: usize) -> f32 {
        self.biases[layer][neuron]
    }

    fn layers_count(&self) -> usize {
        self.biases.len()
    }

    fn neurons_count(&self, layer: usize) -> usize {
        self.biases[layer].len()
    }

    fn connections_count(&self, layer: usize, neuron: usize) -> usize {
        self.weights[layer][neuron].len()
    }

    fn activation(net: f32) -> f32 {
        1.0 / (1.0 + f32::exp(-net))
    }

    fn calculate_output(&self, input: Vec<f32>) -> Vec<f32> {
        let mut neurons: Vec<Vec<f32>> = vec![vec![0.0, 0.0, 0.0], vec![0.0, 0.0]];

        for n_layer in 0..self.layers_count() {
            for n_neuron in 0..self.neurons_count(n_layer) {
                neurons[n_layer][n_neuron] = self.get_bias(n_layer, n_neuron);
                for n_connection in 0..self.connections_count(n_layer, n_neuron) {
                    neurons[n_layer][n_neuron] +=
                        input[n_connection] * self.get_weight(n_layer, n_neuron, n_connection);
                }

                neurons[n_layer][n_neuron] = NeuralNetwork::activation(neurons[n_layer][n_neuron]);
            }
        }

        neurons[1].clone()
    }
}
