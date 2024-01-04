import torch
import warnings

device = "cuda" if torch.cuda.is_available() else "cpu" if torch.backends.mps.is_available() else "cpu"
print(device)


class Net(torch.nn.Module):
    def __init__(self) -> None:
        super().__init__()
        self.common = torch.nn.Sequential(
            torch.nn.Conv2d(
                in_channels=2, 
                out_channels=6, 
                kernel_size=5,
                padding="same"
            ),
            torch.nn.ReLU(),
            torch.nn.Conv2d(
                in_channels=6, 
                out_channels=6, 
                kernel_size=5,
                padding="same" 
            ),
            torch.nn.ReLU(),
        )
        self.value = torch.nn.Sequential(
            torch.nn.Flatten(),
            torch.nn.Linear(in_features=336, out_features=30),
            torch.nn.ReLU(),
            torch.nn.Linear(in_features=30, out_features=1)
        )
        self.policy_log = torch.nn.Sequential(
            torch.nn.Conv2d(
                in_channels=6, 
                out_channels=2, 
                kernel_size=5,
                padding="same" 
            ),
            torch.nn.ReLU(),
            torch.nn.Conv2d(
                in_channels=2, 
                out_channels=1, 
                kernel_size=5,
                padding="same" 
            ),
        )

    def forward(self, x):
        common = self.common(x)
        value = self.value(common)
        policy_log = self.policy_log(common)
        policy_exp = torch.exp(policy_log)
        policy_exp_sum = torch.sum(policy_exp, dim=(2, 3)).unsqueeze(2).unsqueeze(3)
        policy = torch.div(policy_exp, policy_exp_sum)
        return value, policy


model = Net().to(device)
print(model)
x = torch.randn((5, 2, 8, 7)).to(device)
y = model(x)
print(y)

torch_script_graph, unconvertible_ops = torch.onnx.utils.unconvertible_ops(
    model, x, opset_version=16
)

print(unconvertible_ops, "unconv")
torch.onnx.export(
    model,
    x,
    "test.onnx.pb",
    export_params=True,
    opset_version=16,
    do_constant_folding=True,
    input_names=["input"],
    output_names=["output"],
    dynamic_axes={
        "input": {0: "batch_size"},
        "output": {0: "batch_size"}
    }
)