from copy import deepcopy
import csv
import alpha_noah
import record_weighting_functions
import visits_weighting_functions

# Working state format is a 2D array of bytes with 0 for unoccupied, 1 for first player's standard piece, 11 for first player's double piece,
# 2 for the second player's piece, and 2 for the second player's double piece

# Each state hashes to a maximum of 25 bytes - a byte to represent who's turn just finished (just 0 or 1) and one byte per piece up to 24 pieces.
# Each piece is hashed to use two bits to represent its type:
# - 00 for first player standard
# - 01 for first player double
# - 10 for second player standard
# - 11 for second player double
# The other six bits come afterwards and are used to represent the location on the 8x8 board (2^6 = 64 = 8x8)

PLAYER_MARKERS = ['r', 'b']
PLAYER_MARKERS_SET = set(PLAYER_MARKERS)
PLAYER_MARKERS_INDEX_LOOKUP = {}
for i in range(len(PLAYER_MARKERS)):
    PLAYER_MARKERS_INDEX_LOOKUP[PLAYER_MARKERS[i]] = i

STATE_RECORD_WINS_INDEX = 0
STATE_RECORD_LOSSES_INDEX = 1
STATE_RECORD_DRAWS_INDEX = 2

initial_state = [['a', 'b', 'c'], ['d', 'e', 'f'], ['g', 'h', 'i']]
state_records = {}


def get_responsible_player_id_from_state_hash(state_hash):


def get_state_record(state_hash):
    if state_hash in state_records:
        state_record = state_records[state_hash]
        return alpha_noah.create_state_record(
            state_record[STATE_RECORD_WINS_INDEX],
            state_record[STATE_RECORD_LOSSES_INDEX],
            state_record[STATE_RECORD_DRAWS_INDEX])
    return None


def update_state_record(state_hash, did_win, did_draw):
    # let's just not let one player ever learn
    # if state_hash[0] == PLAYER_MARKERS[1]:
    #     return

    # no players ever learn!
    # return

    if state_hash in state_records:
        state_records[state_hash][STATE_RECORD_WINS_INDEX] += 1 if did_win else 0
        state_records[state_hash][STATE_RECORD_LOSSES_INDEX] += 1 if not did_win and not did_draw else 0
        state_records[state_hash][STATE_RECORD_DRAWS_INDEX] += 1 if did_draw else 0
    else:
        state_records[state_hash] = [0, 0, 0]


def compact_hash_state(player_number, state):
    state_raw_value = 0
    ternary_digit_multiplier = 1  # think of this as 10^n where 10 is in base-3
    for i in range(len(state)):
        for j in range(len(state)):
            location_value = 0
            if state[i][j] in PLAYER_MARKERS_INDEX_LOOKUP:
                location_value = PLAYER_MARKERS_INDEX_LOOKUP[state[i][j]] + 1
            state_raw_value += location_value * ternary_digit_multiplier
            ternary_digit_multiplier *= 3

    state_hash = PLAYER_MARKERS[player_number] + '-' + hex(state_raw_value)[2:]
    return state_hash


def intelligibly_hash_state(player_number, state):
    state_hash = PLAYER_MARKERS[player_number] + '-'
    for i in range(len(state)):
        for j in range(len(state)):
            state_hash += state[i][j]

    return state_hash


def find_available_states(player_number, current_state):
    player_marker = PLAYER_MARKERS[player_number]

    available_states = []
    for i in range(len(current_state)):
        for j in range(len(current_state[i])):
            if current_state[i][j] not in PLAYER_MARKERS_SET:
                # the space is empty
                available_state = deepcopy(current_state)
                available_state[i][j] = player_marker
                available_states.append(available_state)

    return available_states


def is_draw_state(player_number, current_state):
    available_states = find_available_states(player_number, current_state)
    is_draw_state = len(available_states) == 0
    return is_draw_state


def is_win_state(player_number, current_state):
    player_marker = PLAYER_MARKERS[player_number]
    winning_player = checkWin(current_state)
    if winning_player == player_marker:
        return True


if __name__ == '__main__':
    num_games = int(input('Enter number of games to simulate: '))

    should_use_custom_weights = input(
        'Do you want to specify the state evaluation weights? ').strip().lower()[:1] == 'y'
    wins_weight = 10 if not should_use_custom_weights else int(
        input('Enter weight of wins for state evaluation: '))
    losses_weight = -10 if not should_use_custom_weights else int(
        input('Enter weight of losses for state evaluation: '))
    draws_weight = 5 if not should_use_custom_weights else int(
        input('Enter weight of draws for state evaluation: '))

    if input('Should learning data be loaded from file? ').strip().lower()[:1] == 'y':
        saved_state_records_file_path = input('Path to learning data file: ')
        with open(saved_state_records_file_path, 'r', newline='') as saved_state_records_csv:
            saved_state_records_reader = csv.reader(saved_state_records_csv)
            for row in saved_state_records_reader:
                state_records[row[0]] = {
                    'wins_count': int(row[1]),
                    'losses_count': int(row[2]),
                    'draws_count': int(row[3])
                }

    print('running ' + str(num_games) + ' games of tic tac toe')

    win_counts_by_player_number = [0, 0]
    for _ in range(num_games):
        winning_player = alpha_noah.execute_standard_turn_based_game(
            initial_state,
            2,  # num_players
            get_state_record,
            update_state_record,
            compact_hash_state,
            find_available_states,
            record_weighting_functions.weighted_sum(
                wins_weight, losses_weight, draws_weight),
            visits_weighting_functions.one,
            is_draw_state,
            is_win_state
        )
        if winning_player >= 0:
            win_counts_by_player_number[winning_player] += 1

    print('Final results! The x player won ' +
          str(win_counts_by_player_number[0]) + ' games and the o player won ' + str(win_counts_by_player_number[1]) + ' games')

    if input('Should learning data be saved to a file? ').strip().lower()[:1] == 'y':
        dest_state_records_file_path = input('Path to save learning data to: ')
        with open(dest_state_records_file_path, 'w', newline='') as dest_state_records_csv:
            dest_state_records_writer = csv.writer(dest_state_records_csv)
            for state_hash in state_records:
                state_record = state_records[state_hash]
                dest_state_records_writer.writerow([
                    state_hash,
                    state_record[STATE_RECORD_WINS_INDEX],
                    state_record[STATE_RECORD_LOSSES_INDEX],
                    state_record[STATE_RECORD_DRAWS_INDEX],
                ])
