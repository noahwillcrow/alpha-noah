use crate::games::checkers::internal::*;
use crate::games::checkers::GameStateType as CheckersGameState;
use crate::traits::{GameStateDeserializer, GameStateSerializer};

/// It is important to note that there's something like 10^20 possible board positions in Checkers / Draught.
/// So, in order to represent all states, at least ceil(log_2(10^20)) bits are necessary, which comes out to 67 bits.
/// This implementation is far less space-efficient as it will take up to 5 + (24*8) = 197 bits to store information,
/// which then is rounded up to the nearest byte for a total of 200 bits.
/// The length of the hash is roughly proportional to the number of pieces on the board, so the average hash length is a complex thing to calculate.
/// Assuming the inaccurate number of 25 bytes per state, it would take rougly 2.5e12 GB to store every single hash possible to represent all 10^20 states.
/// This implementation assumes that that's not a feasible number of states to actually explore in one iteration of the program.
/// So exactly how does the hashing work here?
/// Each state hashes to a maximum of 25 bytes:
/// - The first byte tracks the player who last moved in the two left-most bits (11 for no player responsible)
/// and the number of pieces on the board in the five right-most bits
/// - The rest of the bytes each are one byte per piece up to 24 pieces as described below
/// Each piece is hashed to use the first two bits to represent its type:
/// - 00 for first player standard
/// - 01 for first player double
/// - 10 for second player standard
/// - 11 for second player double
/// The other six bits come afterwards and are used to represent the location on the 8x8 board (2^6 = 64 = 8x8)
pub struct ByteArrayGameStateSerializer {}

impl GameStateSerializer<CheckersGameState, Vec<u8>> for ByteArrayGameStateSerializer {
    fn serialize_game_state(
        &self,
        responsible_player_index: i32,
        game_state: &CheckersGameState,
    ) -> Vec<u8> {
        return serialize_game_state(responsible_player_index, game_state);
    }
}

pub struct ByteArrayGameStateDeserializer {}
impl GameStateDeserializer<CheckersGameState, Vec<u8>> for ByteArrayGameStateDeserializer {
    fn deserialize_game_state(&self, serialized_game_state: &Vec<u8>) -> (i32, CheckersGameState) {
        let first_byte = serialized_game_state[0];
        let responsible_player_bits = first_byte >> 6;
        let number_of_pieces = first_byte & 0b000_11111;
        if serialized_game_state.len() != (number_of_pieces + 1) as usize {
            panic!("Cannot deserialize invalid serialized checkers game state - expected a total of {} bytes, got {}.", number_of_pieces + 1, serialized_game_state.len());
        }

        let responsible_player_index = if responsible_player_bits == 0b000000_11 {
            -1
        } else {
            responsible_player_bits as i32
        };

        let mut game_state: [[u8; 8]; 8] = [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ];

        for i in 0..number_of_pieces {
            let piece_byte = serialized_game_state[i as usize + 1];

            let piece_type_code = (piece_byte & 0b11_000000) >> 6;

            let piece_location = piece_byte & 0b00_111111;
            let row_index = (piece_location / 8) as usize;
            let col_index = (piece_location % 8) as usize;

            match piece_type_code {
                0 => game_state[row_index][col_index] = FIRST_PLAYER_SINGLE_PIECE_VALUE,
                1 => game_state[row_index][col_index] = FIRST_PLAYER_DOUBLE_PIECE_VALUE,
                2 => game_state[row_index][col_index] = SECOND_PLAYER_SINGLE_PIECE_VALUE,
                3 => game_state[row_index][col_index] = SECOND_PLAYER_DOUBLE_PIECE_VALUE,
                _ =>  panic!(
                    "Somehow ended up with a u8 value greater than 3 even though it should have only been two bits?",
                ),
            }
        }

        return (responsible_player_index, game_state);
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_for_invertibility() {
        let serializer = ByteArrayGameStateSerializer {};
        let deserializer = ByteArrayGameStateDeserializer {};

        let test_game_states: Vec<(i32, CheckersGameState)> = vec![
            (
                -1,
                [
                    [0, 1, 0, 1, 0, 1, 0, 1],
                    [1, 0, 1, 0, 1, 0, 1, 0],
                    [0, 1, 0, 1, 0, 1, 0, 1],
                    [0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 0, 0, 0],
                    [2, 0, 2, 0, 2, 0, 2, 0],
                    [0, 2, 0, 2, 0, 2, 0, 2],
                    [2, 0, 2, 0, 2, 0, 2, 0],
                ],
            ),
            (
                0,
                [
                    [0, 22, 0, 1, 0, 1, 0, 1],
                    [1, 0, 1, 0, 1, 0, 1, 0],
                    [0, 1, 0, 1, 0, 1, 0, 1],
                    [0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 0, 0, 0],
                    [2, 0, 2, 0, 2, 0, 2, 0],
                    [0, 2, 0, 2, 0, 2, 0, 2],
                    [2, 0, 2, 0, 2, 0, 11, 0],
                ],
            ),
            (
                1,
                [
                    [0, 22, 0, 1, 0, 1, 0, 1],
                    [1, 0, 1, 0, 1, 0, 1, 0],
                    [0, 1, 0, 1, 0, 1, 0, 1],
                    [0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 0, 0, 0],
                    [2, 0, 2, 0, 2, 0, 2, 0],
                    [0, 2, 0, 2, 0, 2, 0, 2],
                    [2, 0, 2, 0, 2, 0, 11, 0],
                ],
            ),
        ];

        for (responsible_player_index, game_state) in test_game_states.iter() {
            let (deserialized_responsible_player_index, deserialized_game_state) = deserializer
                .deserialize_game_state(
                    &serializer.serialize_game_state(*responsible_player_index, game_state),
                );

            assert_eq!(
                *responsible_player_index,
                deserialized_responsible_player_index
            );
            assert_eq!(*game_state, deserialized_game_state);
        }
    }
}
