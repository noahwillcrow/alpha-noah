use crate::games::checkers::GameStateType as CheckersGameState;
use tch::Tensor;

#[allow(dead_code)]
pub fn transform_game_state_to_tensor(
    _responsible_player_index: i32,
    game_state: &CheckersGameState,
) -> Tensor {
    let mut vectorized_game_state: Vec<Vec<f32>> = vec![];
    for row in game_state {
        let mut row_vector: Vec<f32> = vec![];

        for value in row {
            row_vector.push(*value as f32);
        }

        vectorized_game_state.push(row_vector);
    }

    return Tensor::of_slice2(&vectorized_game_state);
}
