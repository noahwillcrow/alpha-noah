import random

# key pieces:
# - state records (map<state hash, { wins: int, losses: int }>)
# - state hashing function
# - available child states
# - weighing function based on record (weigh_record(wins, losses))
# - weighing function based on number of recorded appearances (weigh_visits(state_visits, min_available_visits, max_available_visits))

STATE_RECORD_LOSSES_INDEX = 0
STATE_RECORD_WINS_INDEX = 1
STATE_RECORD_DRAWS_INDEX = 2

def create_state_record(num_wins, num_losses, num_draws):
    return [num_losses, num_wins, num_draws]

def decide_next_state(
    current_player,
    get_state_record,
    hash_state,
    available_states,
    weigh_record,
    weigh_visits
):
    min_available_visits = 2**31
    max_available_visits = 0

    available_state_records_by_hash = {}

    for available_state in available_states:
        available_state_hash = hash_state(current_player, available_state)
        available_state_record = get_state_record(available_state_hash)

        if available_state_record is not None:
            available_state_records_by_hash[available_state_hash] = available_state_record

            num_losses = available_state_record[STATE_RECORD_LOSSES_INDEX]
            num_wins = available_state_record[STATE_RECORD_WINS_INDEX]
            num_draws = available_state_record[STATE_RECORD_DRAWS_INDEX]
            num_visits = num_losses + num_wins + num_draws

            min_available_visits = min(min_available_visits, num_visits)
            max_available_visits = max(max_available_visits, num_visits)

    available_state_weights = [0 for i in range(len(available_states))]

    for i in range(len(available_states)):
        num_losses = 0
        num_wins = 0
        num_draws = 0
        num_visits = 0
        
        available_state_hash = hash_state(current_player, available_state)
        if available_state_hash in available_state_records_by_hash:
            available_state_record = available_state_records_by_hash[available_state_hash]
            num_losses = available_state_record[STATE_RECORD_LOSSES_INDEX]
            num_wins = available_state_record[STATE_RECORD_WINS_INDEX]
            num_draws = available_state_record[STATE_RECORD_DRAWS_INDEX]
            num_visits = num_losses + num_wins + num_draws

        record_weight = weigh_record(num_wins, num_losses, num_draws)
        visits_weight = weigh_visits(num_visits, min_available_visits, max_available_visits)
        combined_weight = record_weight + visits_weight
        available_state_weights[i] = combined_weight

    chosen_state = random.choices(available_states, available_state_weights)[0]
    return chosen_state

def update_state_records(
    update_state_record,
    state_paths_by_player,
    winning_player
):
    did_draw = winning_player == -1
    for player_number in range(len(state_paths_by_player)):
        did_win = player_number == winning_player
        for state_hash in state_paths_by_player[player_number]:
            update_state_record(state_hash, did_win, did_draw)

def execute_standard_turn_based_game(
    initial_state,
    num_players,
    get_state_record,
    update_state_record,
    hash_state,
    find_available_states,
    weigh_record,
    weigh_visits,
    is_draw_state,
    is_win_state
):
    # assume valid input right now

    # stores series of hashes of each state that the individual players caused
    state_paths_by_player = [[] for i in range(num_players)]

    current_player = 0
    current_state = initial_state
    while True:
        available_states = find_available_states(current_player, current_state)
        new_state = decide_next_state(
            current_player,
            get_state_record,
            hash_state,
            available_states,
            weigh_record,
            weigh_visits
        )
        state_paths_by_player[current_player].append(hash_state(current_player, new_state))

        if is_draw_state(current_player, new_state):
            # for now, count draws as losses - probably not a good idea though
            update_state_records(update_state_record, state_paths_by_player, -1)
            return -1

        for potential_winner in range(num_players):
            if is_win_state(potential_winner, new_state):
                update_state_records(update_state_record, state_paths_by_player, potential_winner)
                return potential_winner

        current_player = (current_player + 1) % num_players
        current_state = new_state

if __name__ == '__main__':
    raise RuntimeError('Not meant to execute this file directly')