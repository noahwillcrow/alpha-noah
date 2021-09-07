use tch::{nn, Tensor};

#[derive(Debug)]
pub struct TorchNet {
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
    conv3: nn::Conv2D,
    fc1: nn::Linear,
    fc2: nn::Linear,
}

impl TorchNet {
    #[allow(dead_code)]
    pub fn new(vs: &nn::Path) -> TorchNet {
        return TorchNet {
            conv1: nn::conv2d(vs, 5, 32, 3, Default::default()),
            conv2: nn::conv2d(vs, 32, 64, 3, Default::default()),
            conv3: nn::conv2d(vs, 64, 64, 3, Default::default()),
            fc1: nn::linear(vs, 256, 512, Default::default()),
            fc2: nn::linear(vs, 512, 1, Default::default()),
        };
    }
}

impl nn::Module for TorchNet {
    fn forward(&self, tensor: &Tensor) -> Tensor {
        return tensor
            .view([1, -1, 8, 8])
            .apply(&self.conv1)
            .relu()
            .apply(&self.conv2)
            .relu()
            .apply(&self.conv3)
            .relu()
            .view([1, -1])
            .apply(&self.fc1)
            .relu()
            .apply(&self.fc2);
    }
}
