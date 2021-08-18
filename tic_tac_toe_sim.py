# this part was copied from https://stackoverflow.com/a/39923094
import numpy as np

def checkRows(board):
    for row in board:
        if len(set(row)) == 1:
            return row[0]
    return 0

def checkDiagonals(board):
    if len(set([board[i][i] for i in range(len(board))])) == 1:
        return board[0][0]
    if len(set([board[i][len(board)-i-1] for i in range(len(board))])) == 1:
        return board[0][len(board)-1]
    return 0

def checkWin(board):
    #transposition to check rows, then columns
    for newBoard in [board, np.transpose(board)]:
        result = checkRows(newBoard)
        if result:
            return result
    return checkDiagonals(board)

# end stackoverflow thing

from copy import deepcopy
import alpha_noah
import record_weighting_functions
import visits_weighting_functions

# state format is a 2D array with 'x's and 'o's.
# To satisfy the above win condition checker, empty values will be from [['a', 'b', 'c'], ['d', 'e', 'f'], ['g', 'h', 'i']]

PLAYER_MARKERS = ['x', 'o']
PLAYER_MARKERS_SET = set(PLAYER_MARKERS)
PLAYER_MARKERS_INDEX_LOOKUP = {}
for i in range(len(PLAYER_MARKERS)):
    PLAYER_MARKERS_INDEX_LOOKUP[PLAYER_MARKERS[i]] = i

initial_state = [['a', 'b', 'c'], ['d', 'e', 'f'], ['g', 'h', 'i']]
state_records = {}

def get_state_record(state_hash):
    if state_hash in state_records:
        state_record = state_records[state_hash]
        return alpha_noah.create_state_record(
            state_record['wins_count'],
            state_record['losses_count'],
            state_record['draws_count'])
    return None

def update_state_record(state_hash, did_win, did_draw):
    # let's just not let one player ever learn
    # if state_hash[0] == PLAYER_MARKERS[1]:
    #     return

    if state_hash in state_records:
        state_records[state_hash]['wins_count'] += 1 if did_win else 0
        state_records[state_hash]['losses_count'] += 1 if not did_win and not did_draw else 0
        state_records[state_hash]['draws_count'] += 1 if did_draw else 0
    else:
        state_records[state_hash] = {
            'wins_count': 0,
            'losses_count': 0,
            'draws_count': 0
        }

def compact_hash_state(player_number, state):
    state_raw_value = 0
    ternary_digit_multiplier = 1 # think of this as 10^n where 10 is in base-3
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
    num_games = 10000
    print('running ' + str(num_games) + ' games of tic tac toe')

    win_counts_by_player_number = [0, 0]
    for _ in range(num_games):
        winning_player = alpha_noah.execute_standard_turn_based_game(
            initial_state,
            2, # num_players
            get_state_record,
            update_state_record,
            compact_hash_state,
            find_available_states,
            record_weighting_functions.weighted_sum(10, -10, 5),
            visits_weighting_functions.one,
            is_draw_state,
            is_win_state
        )
        if winning_player >= 0:
            win_counts_by_player_number[winning_player] += 1
        
    print('Final results! The x player won ' + str(win_counts_by_player_number[0]) + ' games and the o player won ' + str(win_counts_by_player_number[1]) + ' games')