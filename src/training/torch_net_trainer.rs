use crate::structs::GameReport;
use crate::traits::{
    BasicGameState, BasicSerializedGameState, GameReportsProcessor, GameStateDeserializer,
    PendingUpdatesManager,
};
use std::cell::{Cell, RefCell};
use std::thread;
use tch::{nn, nn::OptimizerConfig, Tensor};

const MAX_PENDING_UPDATES_COUNT: u32 = 1000;

pub struct TorchNetTrainer<
    'a,
    GameState: BasicGameState,
    SerializedGameState: BasicSerializedGameState,
> {
    file_name: &'a str,
    game_state_deserializer: &'a dyn GameStateDeserializer<GameState, SerializedGameState>,
    optimizer_ref_cell: RefCell<nn::Optimizer<nn::Adam>>,
    pending_updates_count_cell: Cell<u32>,
    torch_net: &'a dyn nn::Module,
    transform_game_state_to_tensor: &'a dyn Fn(i32, &GameState) -> Tensor,
    var_store: &'a nn::VarStore,
}

impl<'a, GameState: BasicGameState, SerializedGameState: BasicSerializedGameState>
    TorchNetTrainer<'a, GameState, SerializedGameState>
{
    pub fn new(
        file_name: &'a str,
        game_state_deserializer: &'a dyn GameStateDeserializer<GameState, SerializedGameState>,
        torch_net: &'a dyn nn::Module,
        var_store: &'a nn::VarStore,
        transform_game_state_to_tensor: &'a dyn Fn(i32, &GameState) -> Tensor,
    ) -> TorchNetTrainer<'a, GameState, SerializedGameState> {
        return TorchNetTrainer {
            file_name: file_name,
            game_state_deserializer: game_state_deserializer,
            optimizer_ref_cell: RefCell::new(nn::Adam::default().build(var_store, 1e-4).unwrap()),
            pending_updates_count_cell: Cell::new(0),
            torch_net: torch_net,
            transform_game_state_to_tensor: transform_game_state_to_tensor,
            var_store: var_store,
        };
    }
}

impl<'a, GameState: BasicGameState, SerializedGameState: BasicSerializedGameState>
    GameReportsProcessor<SerializedGameState, ()>
    for TorchNetTrainer<'a, GameState, SerializedGameState>
{
    fn process_game_report(
        &self,
        game_report: &mut GameReport<SerializedGameState>,
    ) -> Result<(), ()> {
        let did_draw = game_report.winning_player_index == -1;

        while let Some(game_state_update) = game_report.game_state_updates.pop() {
            let did_win =
                game_state_update.responsible_player_index == game_report.winning_player_index;

            let (_, game_state) = self
                .game_state_deserializer
                .deserialize_game_state(&game_state_update.new_serialized_game_state);
            let game_state_tensor = (self.transform_game_state_to_tensor)(
                game_state_update.responsible_player_index,
                &game_state,
            );

            let result_value = if did_win {
                1
            } else if did_draw {
                0
            } else {
                -1
            };
            let result_tensor = Tensor::of_slice(&[result_value as f32]).view([1, 1]);

            let prediction_tensor = self.torch_net.forward(&game_state_tensor);
            let loss_tensor = (result_tensor - prediction_tensor).pow(2);
            self.optimizer_ref_cell
                .borrow_mut()
                .backward_step(&loss_tensor);
        }

        self.pending_updates_count_cell
            .set(self.pending_updates_count_cell.get() + 1);
        if self.pending_updates_count_cell.get() >= MAX_PENDING_UPDATES_COUNT {
            self.try_commit_pending_updates_in_background(0);
        }

        return Ok(());
    }
}

impl<'a, GameState: BasicGameState, SerializedGameState: BasicSerializedGameState>
    PendingUpdatesManager for TorchNetTrainer<'a, GameState, SerializedGameState>
{
    fn try_commit_pending_updates_in_background(
        &self,
        _max_number_to_commit: usize,
    ) -> std::thread::JoinHandle<()> {
        match self.var_store.save(self.file_name) {
            Ok(_) => {
                self.pending_updates_count_cell.set(0);
            }
            Err(_) => (),
        };

        return thread::spawn(|| {});
    }
}
