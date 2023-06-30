import torch
import warnings

device = "cuda" if torch.cuda.is_available() else "cpu" if torch.backends.mps.is_available() else "cpu"
print(device)


class Net(torch.nn.Module):
    def __init__(self) -> None:
        super().__init__()
        self.flatten = torch.nn.Flatten()
        self.ll = torch.nn.Sequential(
            torch.nn.Linear(6, 5),
            torch.nn.ReLU(),
            torch.nn.Linear(5, 2),
        )
    def forward(self, x):
        x = self.flatten(x)
        logits = self.ll(x)
        return logits


model = Net().to(device)
print(model)
x = torch.Tensor([[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]]).to(device)
y = model(x)
print(y)

torch.onnx.export(
    model,
    x,
    "test.onnx.pb",
    export_params=True,
    opset_version=14,
    do_constant_folding=True,
    input_names=["input"],
    output_names=["output"],
    dynamic_axes={
        "input": {0: "batch_size"},
        "output": {0: "batch_size"}
    }
)