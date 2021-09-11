use crate::traits::{BasicGameState, GameStateWeightsCalculator};
use tch::{nn, Device, Tensor};

pub struct CnnGameStateWeightsCalculator<'a, GameState: BasicGameState> {
    device: Device,
    torch_net: &'a dyn nn::Module,
    transform_game_state_to_tensor: &'a dyn Fn(i32, &GameState) -> Tensor,
    //optimizer: nn::Optimizer<nn::Adam>,
}

impl<'a, GameState: BasicGameState> CnnGameStateWeightsCalculator<'a, GameState> {
    #[allow(dead_code)]
    pub fn new(
        device: Device,
        torch_net: &'a dyn nn::Module,
        transform_game_state_to_tensor: &'a dyn Fn(i32, &GameState) -> Tensor,
    ) -> CnnGameStateWeightsCalculator<'a, GameState> {
        return CnnGameStateWeightsCalculator {
            device: device,
            torch_net: torch_net,
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
            let result_tensor = self.torch_net.forward(&game_state_tensor.to(self.device));
            let weight = result_tensor.double_value(&[0]) as f32;

            weights.push(weight);
        }

        return weights;
    }
}
