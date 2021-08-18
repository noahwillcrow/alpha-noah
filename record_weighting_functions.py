def wins_plus_1_ratio(num_wins, num_losses, num_draws):
    return (num_wins + 1) / (num_wins + num_losses + num_draws + 1)

def wins_minus_losses_floored(num_wins, num_losses, num_draws):
    return max(1, num_wins - num_losses)

def weighted_sum(wins_weight, losses_weight, draws_weight):
    def result(num_wins, num_losses, num_draws):
        return max(1, num_wins * wins_weight + num_losses * losses_weight + num_draws * draws_weight)
    return result