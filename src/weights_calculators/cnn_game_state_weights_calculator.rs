use crate::traits::{BasicGameState, GameStateWeightsCalculator};
use tch::{nn, nn::Module, Device, Tensor};

#[derive(Debug)]
struct Net {
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
    conv3: nn::Conv2D,
    fc1: nn::Linear,
    fc2: nn::Linear,
}

impl Net {
    #[allow(dead_code)]
    fn new(vs: &nn::Path) -> Net {
        return Net {
            conv1: nn::conv2d(vs, 5, 32, 3, Default::default()),
            conv2: nn::conv2d(vs, 32, 64, 3, Default::default()),
            conv3: nn::conv2d(vs, 64, 64, 3, Default::default()),
            fc1: nn::linear(vs, 256, 512, Default::default()),
            fc2: nn::linear(vs, 512, 1, Default::default()),
        };
    }
}

impl nn::Module for Net {
    fn forward(&self, tensor: &Tensor) -> Tensor {
        let mut in_progress = tensor.view([1, -1, 8, 8]);
        in_progress = in_progress.apply(&self.conv1).relu();
        println!("conv1 result: {:?}", in_progress);
        // in_progress = in_progress.max_pool2d_default(2);
        // println!("max pool result: {:?}", in_progress);
        in_progress = in_progress.apply(&self.conv2).relu();
        println!("conv2 result: {:?}", in_progress);
        in_progress = in_progress.apply(&self.conv3).relu();
        println!("conv3 result: {:?}", in_progress);
        in_progress = in_progress.view([1, -1]);
        println!("view result: {:?}", in_progress);
        in_progress = in_progress.apply(&self.fc1).relu();
        println!("fc1 result: {:?}", in_progress);
        in_progress = in_progress.apply(&self.fc2);
        println!("fc2 result: {:?}", in_progress);
        return in_progress;
    }
}

pub struct CnnGameStateWeightsCalculator<'a, GameState: BasicGameState> {
    network_module: Net, // &'a dyn nn::Module,
    transform_game_state_to_tensor: &'a dyn Fn(i32, &GameState) -> Tensor,
    //optimizer: nn::Optimizer<nn::Adam>,
}

impl<'a, GameState: BasicGameState> CnnGameStateWeightsCalculator<'a, GameState> {
    #[allow(dead_code)]
    pub fn new(
        //network_module: &'a dyn nn::Module,
        transform_game_state_to_tensor: &'a dyn Fn(i32, &GameState) -> Tensor,
    ) -> CnnGameStateWeightsCalculator<'a, GameState> {
        let vs = nn::VarStore::new(Device::cuda_if_available());
        return CnnGameStateWeightsCalculator {
            network_module: Net::new(&vs.root()),
            transform_game_state_to_tensor: transform_game_state_to_tensor,
            // net: Net::new(&var_store.root()),
            //optimizer: nn::Adam::default().build(&var_store, 1e-4).unwrap(),
        };
    }
}

impl<'a, GameState: BasicGameState> GameStateWeightsCalculator<GameState>
    for CnnGameStateWeightsCalculator<'a, GameState>
{
    fn weigh_game_states(
        &self,
        responsible_player_index: i32,
        game_states: &Vec<GameState>,
    ) -> Vec<f32> {
        let mut weights: Vec<f32> = vec![];

        for game_state in game_states.iter() {
            let game_state_tensor =
                (self.transform_game_state_to_tensor)(responsible_player_index, game_state);
            let result_tensor = self.network_module.forward(&game_state_tensor);
            let weight = result_tensor.double_value(&[0]) as f32;

            weights.push(weight);
        }

        return weights;
    }
}
