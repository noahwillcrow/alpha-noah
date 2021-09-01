use crate::core::state_record::StateRecord;
use crate::core::state_record_provider::StateRecordProvider;
use std::collections::HashSet;

// Working state format is a 2D array of bytes with 0 for unoccupied, 1 for first player's standard piece, 11 for first player's double piece,
// 2 for the second player's piece, and 2 for the second player's double piece

// It is important to note that there's something like 10^20 possible board positions in Checkers / Draught.
// So, in order to represent all states, at least ceil(log_2(10^20)) bits are necessary, which comes out to 67 bits.
// This implementation is far less space-efficient as it will take up to 5 + (24*8) = 197 bits to store information,
// which then is rounded up to the nearest byte for a total of 200 bits.
// The length of the hash is proportional to the number of pieces on the board, so the average hash length is a complex thing to calculate.
// Assuming the inaccurate number of 25 bytes per state, it would take rougly 2.5e12 GB to store every single hash possible to represent all 10^20 states.
// This implementation assumes that that's not a feasible number of states to actually explore in one iteration of the program.
// So exactly how does the hashing work here?
// Each state hashes to a maximum of 25 bytes:
// - The first byte tracks the player who last moved and the number of pieces on the board
// - The rest of the bytes each are one byte per piece up to 24 pieces as described below
// Each piece is hashed to use the first two bits to represent its type:
// - 00 for first player standard
// - 01 for first player double
// - 10 for second player standard
// - 11 for second player double
// The other six bits come afterwards and are used to represent the location on the 8x8 board (2^6 = 64 = 8x8)

const MAX_ROW: i8 = 7;
const MAX_COL: i8 = 7;

struct MoveSearchParameters {
    pub single_piece_value: u8,
    pub double_piece_value: u8,
    pub single_piece_available_directions: [(i8, i8); 2],
    pub double_piece_available_directions: [(i8, i8); 4],
    pub double_row: usize,
}

pub type CheckersState = Vec<Vec<u8>>;

pub fn get_terminal_state(current_player_index: i32, current_state: &CheckersState) -> Option<i32> {
    let move_search_params = get_player_specific_move_search_parameters(current_player_index);
    let mut owned_pieces_info: Vec<((usize, usize), &[(i8, i8)])> = vec![];
    fill_vector_with_current_player_owned_pieces_info_for_move_search(
        current_state,
        &move_search_params,
        &mut owned_pieces_info,
    );

    let mut available_move_states: Vec<CheckersState> = vec![];

    // see if any simple moves exist for any pieces first since that's cheaper
    for (current_coor, available_directions) in owned_pieces_info.iter() {
        fill_vector_with_available_simple_move_states_for_piece(
            current_state,
            current_coor,
            available_directions,
            move_search_params.single_piece_value,
            move_search_params.double_piece_value,
            move_search_params.double_row,
            &mut available_move_states,
            1,
        );

        if !available_move_states.is_empty() {
            // at least one move exists so this player hasn't lost yet
            return None;
        }
    }

    // no simple moves were found so let's see if we have
    // any capture moves (which are more expensive to find) are available
    for (current_coor, available_directions) in owned_pieces_info.iter() {
        fill_vector_with_available_capture_move_states_for_piece(
            current_player_index,
            current_state,
            current_coor,
            available_directions,
            move_search_params.single_piece_value,
            move_search_params.double_piece_value,
            move_search_params.double_row,
            &mut available_move_states,
            1,
        );

        if !available_move_states.is_empty() {
            // at least one move exists so this player hasn't lost yet
            return None;
        }
    }

    // no available moves have been found
    // so the current player can't move for their coming turn
    // which means they lost
    // so let's return Some(other_player_index) to report that the game has ended and the other player has won
    let other_player_index = (current_player_index + 1) % 2;
    return Some(other_player_index);
}

pub fn hash_state(responsible_player_index: i32, current_state: &CheckersState) -> Vec<u8> {
    let mut number_of_pieces: u8 = 0;
    let mut state_hash_bytes = vec![0 as u8];

    for i in 0..current_state.len() {
        for j in 0..current_state.len() {
            let position_value = current_state[i][j];
            if position_value > 0 {
                number_of_pieces += 1;

                let mut piece_byte = (i * j) as u8;

                match position_value {
                    1 => (),
                    11 => piece_byte &= 0b01_00_00_00,
                    2 => piece_byte &= 0b10_00_00_00,
                    22 => piece_byte &= 0b11_00_00_00,
                    _ => panic!(
                        "Encountered illegal position value {} at position ({}, {})",
                        position_value, i, j
                    ),
                }

                state_hash_bytes.push(piece_byte);
            }
        }
    }

    state_hash_bytes[0] = ((responsible_player_index as u8) << 7) + number_of_pieces;

    return state_hash_bytes;
}

fn is_valid_board_coordinate(row: i8, col: i8) -> bool {
    return row >= 0 && row <= MAX_ROW && col >= 0 && col <= MAX_COL;
}

pub struct CheckersBotNerferStateRecordProvider<'a> {
    base_state_record_provider: &'a mut dyn StateRecordProvider<Vec<u8>>,
    nerfed_player_index: i32,
}

impl<'a> CheckersBotNerferStateRecordProvider<'a> {
    pub fn new(
        base_state_record_provider: &'a mut dyn StateRecordProvider<Vec<u8>>,
        nerfed_player_index: i32,
    ) -> CheckersBotNerferStateRecordProvider<'a> {
        return CheckersBotNerferStateRecordProvider {
            base_state_record_provider: base_state_record_provider,
            nerfed_player_index: nerfed_player_index,
        };
    }
}

impl<'a> StateRecordProvider<Vec<u8>> for CheckersBotNerferStateRecordProvider<'a> {
    fn get_state_record(&mut self, state_hash: &Vec<u8>) -> Option<StateRecord> {
        let first_byte = state_hash[0];
        let responsible_player_index = first_byte >> 7;

        if responsible_player_index as i32 == self.nerfed_player_index {
            // don't get to use the learnings here
            return Some(StateRecord::new(0, 0, 0));
        }

        return self
            .base_state_record_provider
            .get_state_record(&state_hash);
    }

    fn update_state_record(&mut self, state_hash: Vec<u8>, did_draw: bool, did_win: bool) {
        self.base_state_record_provider
            .update_state_record(state_hash, did_draw, did_win);
    }
}
