use crate::traits::{BasicGameState, GameStateWeightsCalculator};
use tch::{nn, nn::ModuleT, Tensor};

#[derive(Debug)]
struct Net {
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
    fc1: nn::Linear,
    fc2: nn::Linear,
}

impl Net {
    #[allow(dead_code)]
    fn new(vs: &nn::Path) -> Net {
        return Net {
            conv1: nn::conv2d(vs, 1, 64, 3, Default::default()),
            conv2: nn::conv2d(vs, 64, 128, 3, Default::default()),
            fc1: nn::linear(vs, 4, 8, Default::default()),
            fc2: nn::linear(vs, 8, 1, Default::default()),
        };
    }
}

impl nn::ModuleT for Net {
    fn forward_t(&self, tensor: &Tensor, train: bool) -> Tensor {
        return tensor
            .view([-1, 1, 8, 8])
            .apply(&self.conv1)
            .relu()
            .apply(&self.conv2)
            .relu()
            .apply(&self.fc1)
            .relu()
            .dropout_(0.2, train)
            .apply(&self.fc2);
    }
}

pub struct CnnGameStateWeightsCalculator<'a, GameState: BasicGameState> {
    transform_game_state_to_tensor: &'a dyn Fn(i32, &GameState) -> Tensor,
    net: &'a dyn ModuleT,
    //optimizer: nn::Optimizer<nn::Adam>,
}

impl<'a, GameState: BasicGameState> CnnGameStateWeightsCalculator<'a, GameState> {
    #[allow(dead_code)]
    pub fn new(
        net: &'a dyn ModuleT,
        transform_game_state_to_tensor: &'a dyn Fn(i32, &GameState) -> Tensor,
    ) -> CnnGameStateWeightsCalculator<'a, GameState> {
        // let var_store = nn::VarStore::new(Device::cuda_if_available());

        return CnnGameStateWeightsCalculator {
            transform_game_state_to_tensor: transform_game_state_to_tensor,
            net: net, //Net::new(&var_store.root()),
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
            let net_forward_result = self.net.forward_t(&game_state_tensor, false);
            let weight = net_forward_result.double_value(&[0, 0, 0, 0]) as f32;

            weights.push(weight);
        }

        return weights;
    }
}
