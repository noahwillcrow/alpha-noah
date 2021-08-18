import random

# key pieces:
# - state records (map<state hash, { wins: int, losses: int }>)
# - state hashing function
# - available child states
# - weighing function based on record (weigh_record(wins, losses))
# - weighing function based on number of recorded appearances (weigh_visits(state_visits, min_available_visits, max_available_visits))

STATE_RECORD_LOSSES_INDEX = 1
STATE_RECORD_WINS_INDEX = 1

def decide_next_state(
    state_records,
    hash_state,
    available_states,
    weigh_record,
    weigh_visits
):
    min_available_visits = 2**31
    max_available_visits = 0

    for available_state in available_states:
        available_state_hash = hash_state(available_state)
        if available_state_hash in state_records:
            available_state_record = state_records[available_state_hash]
            num_losses = available_state_record[STATE_RECORD_LOSSES_INDEX]
            num_wins = available_state_record[STATE_RECORD_WINS_INDEX]
            num_visits = num_losses + num_wins

            min_available_visits = min(min_available_visits, num_visits)
            max_available_visits = max(max_available_visits, num_visits)

    available_state_weights = []

    for i in range(len(available_states)):
        num_losses = 0
        num_wins = 0
        num_visits = 0
        
        available_state_hash = hash_state(available_state)
        if available_state_hash in state_records:
            num_losses = available_state_record[STATE_RECORD_LOSSES_INDEX]
            num_wins = available_state_record[STATE_RECORD_WINS_INDEX]
            num_visits = num_losses + num_wins

        record_weight = weigh_record(num_wins, num_losses)
        visits_weight = weigh_visits(num_visits, min_available_visits, max_available_visits)
        combined_weight = record_weight + visits_weight
        available_state_weights[i] = combined_weight

    chosen_state = random.choices(available_states, available_state_weights)
    return chosen_state

if __name__ == '__main__':
    decide_next_state(None, None, None, None, None)