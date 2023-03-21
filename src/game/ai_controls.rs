use bevy::prelude::*;
use rand::Rng;

use super::{eval_shot, GameState};

pub struct AIControlsPlugin;

impl Plugin for AIControlsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AIControls::new());
    }
}

#[derive(Resource)]
pub struct AIControls {
    is_enabled: bool,
    net: NeuralNetwork,
    pull_power: f32,
    pull_angle: f32,
}

impl AIControls {
    fn new() -> Self {
        let mut genetic_algorithm = GeneticAlgorithm::new(1024, 0.5);
        let best_net = genetic_algorithm.get_best(100);

        Self {
            is_enabled: false,
            net: best_net,
            pull_power: 0.0,
            pull_angle: 0.0,
        }
    }

    pub fn enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn set_enabled(&mut self, value: bool) {
        self.is_enabled = value;
    }

    pub fn think(&mut self, game_state: &GameState) {
        let input = vec![game_state.enemy_height, game_state.player_height];
        let output = self.net.calculate_output(input);
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

#[derive(Clone)]
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

    fn mutate(&mut self, rate: f32) {
        for layer in self.weights.iter_mut() {
            for neuron in layer.iter_mut() {
                for weight in neuron.iter_mut() {
                    *weight += rand::thread_rng().gen_range(-1.0..=1.0) * rate;
                }
            }
        }
    }

    fn crossover(&mut self, other: &NeuralNetwork, alpha: f32) {
        for n_layer in 0..self.layers_count() {
            for n_neuron in 0..self.neurons_count(n_layer) {
                for n_connection in 0..self.connections_count(n_layer, n_neuron) {
                    let self_weight = self.get_weight(n_layer, n_neuron, n_connection);
                    let other_weight = other.get_weight(n_layer, n_neuron, n_connection);
                    let average = ((1.0 - alpha) * self_weight) + (alpha * other_weight);
                    self.weights[n_layer][n_neuron][n_connection] = average;
                }
            }
        }
    }

    fn score(&self) -> i32 {
        let mut score = 0;
        for i in 0..=10 {
            let h_self = i as f32 * 0.1;
            for j in 0..=10 {
                let h_other = j as f32 * 0.1;
                let input = vec![h_self, h_other];
                let output = self.calculate_output(input);
                let shot_score = eval_shot(output[0], output[1], h_self, h_other);
                score += shot_score;
            }
        }

        score
    }
}

#[derive(Clone)]
struct Agent {
    net: NeuralNetwork,
    score: i32,
    fitness: f32,
}

struct GeneticAlgorithm {
    pop_size: usize,
    learning_rate: f32,
    current_gen: u32,
    agents: Vec<Agent>,
}

impl GeneticAlgorithm {
    fn new(pop_size: usize, learning_rate: f32) -> Self {
        Self {
            pop_size,
            learning_rate,
            current_gen: 0,
            agents: Vec::new(),
        }
    }

    pub fn get_best(&mut self, generations: u32) -> NeuralNetwork {
        self.current_gen = 0;
        self.init_random();

        while self.current_gen < generations {
            self.calculate_fitness();
            self.new_generation();

            self.current_gen += 1;
        }

        self.calculate_fitness();
        let mut best: NeuralNetwork = self.agents[0].net.clone();
        let mut best_fitness = f32::MIN;
        for agent in self.agents.iter() {
            if agent.fitness > best_fitness {
                best = agent.net.clone();
                best_fitness = agent.fitness;
            }
        }

        best
    }

    fn init_random(&mut self) {
        let mut random_agents: Vec<Agent> = Vec::with_capacity(self.pop_size);
        for _ in 0..self.pop_size {
            random_agents.push(Agent {
                net: NeuralNetwork::new(),
                score: 0,
                fitness: 0.0,
            });
        }

        self.agents = random_agents;
    }

    fn calculate_fitness(&mut self) {
        let mut score_sum = 0;
        for mut agent in self.agents.iter_mut() {
            agent.score = agent.net.score();
            score_sum += agent.score;
        }

        for mut agent in self.agents.iter_mut() {
            agent.fitness = agent.score as f32 / score_sum as f32;
        }
    }

    fn select_parent(&self) -> &Agent {
        let mut selected_index = 0;
        let mut r = rand::thread_rng().gen_range(0.01..=1.0);
        while r > 0.0 {
            r -= self.agents[selected_index].fitness;
            selected_index += 1;
        }

        selected_index -= 1;
        &self.agents[selected_index]
    }

    fn new_generation(&mut self) {
        let mut new_agents: Vec<Agent> = Vec::with_capacity(self.pop_size);
        while new_agents.len() < self.pop_size - 1 {
            let p1 = self.select_parent();
            let p2 = self.select_parent();
            let mut child = p1.clone();
            let alpha = rand::thread_rng().gen_range(0.0..=1.0);

            child.net.crossover(&p2.net, alpha);
            child.net.mutate(self.learning_rate);

            new_agents.push(child);
        }

        self.agents = new_agents;
    }
}
