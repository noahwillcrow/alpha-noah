use crate::games::checkers::internal::*;
use crate::games::checkers::GameStateType as CheckersGameState;
use crate::traits::GameStateSerializer;

/// It is important to note that there's something like 10^20 possible board positions in Checkers / Draught.
/// So, in order to represent all states, at least ceil(log_2(10^20)) bits are necessary, which comes out to 67 bits.
/// This implementation is far less space-efficient as it will take up to 5 + (24*8) = 197 bits to store information,
/// which then is rounded up to the nearest byte for a total of 200 bits.
/// The length of the hash is proportional to the number of pieces on the board, so the average hash length is a complex thing to calculate.
/// Assuming the inaccurate number of 25 bytes per state, it would take rougly 2.5e12 GB to store every single hash possible to represent all 10^20 states.
/// This implementation assumes that that's not a feasible number of states to actually explore in one iteration of the program.
/// So exactly how does the hashing work here?
/// Each state hashes to a maximum of 25 bytes:
/// - The first byte tracks the player who last moved and the number of pieces on the board
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
