def wins_plus_1_ratio(num_wins, num_losses):
    return (num_wins + 1) / (num_wins + num_losses + 1)

def wins_minus_losses_floored(num_wins, num_losses):
    return max(1, num_wins - num_losses)