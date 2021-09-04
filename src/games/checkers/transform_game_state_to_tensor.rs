use crate::games::checkers::internal::*;
use crate::games::checkers::GameStateType as CheckersGameState;
use tch::Tensor;

const ROW_LENGTH: usize = 8;
const COL_LENGTH: usize = 8;
const BOARD_SIZE: usize = ROW_LENGTH * COL_LENGTH;

#[allow(dead_code)]
pub fn transform_game_state_to_tensor(
    responsible_player_index: i32,
    game_state: &CheckersGameState,
) -> Tensor {
    let mut flattened_tensor_values = [0_f32; BOARD_SIZE * 5];

    let first_player_single_pieces_offset = BOARD_SIZE * 0;
    let first_player_double_pieces_offset = BOARD_SIZE * 1;
    let second_player_single_pieces_offset = BOARD_SIZE * 2;
    let second_player_double_pieces_offset = BOARD_SIZE * 3;
    let responsible_player_index_offset = BOARD_SIZE * 4;

    for (row_index, row_vector) in game_state.iter().enumerate() {
        let row_offset = row_index * 8;

        for (column_index, space_value) in row_vector.iter().enumerate() {
            let space_offset = row_offset + column_index;

            match *space_value {
                FIRST_PLAYER_SINGLE_PIECE_VALUE => {
                    flattened_tensor_values[first_player_single_pieces_offset + space_offset] =
                        1_f32
                }
                FIRST_PLAYER_DOUBLE_PIECE_VALUE => {
                    flattened_tensor_values[first_player_double_pieces_offset + space_offset] =
                        1_f32
                }
                SECOND_PLAYER_SINGLE_PIECE_VALUE => {
                    flattened_tensor_values[second_player_single_pieces_offset + space_offset] =
                        1_f32
                }
                SECOND_PLAYER_DOUBLE_PIECE_VALUE => {
                    flattened_tensor_values[second_player_double_pieces_offset + space_offset] =
                        1_f32
                }
                _ => (),
            }

            flattened_tensor_values[responsible_player_index_offset + space_offset] =
                responsible_player_index as f32;
        }
    }

    let flattened_tensor = Tensor::of_slice(&flattened_tensor_values);
    return flattened_tensor.view([5 as i64, ROW_LENGTH as i64, COL_LENGTH as i64]);
}
