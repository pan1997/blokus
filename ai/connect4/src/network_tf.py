
import tensorflow as tf
import onnx
import tf2onnx


class Network(tf.keras.Model):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.common = tf.keras.Sequential([
            tf.keras.layers.Conv2D(6, 5, padding='same', activation='relu'),
        ])
        self.value = tf.keras.Sequential([
            tf.keras.layers.Flatten(),
            tf.keras.layers.Dense(30, activation='relu'),
            tf.keras.layers.Dense(1)
        ])
        self.policy_log = tf.keras.Sequential([
            tf.keras.layers.Conv2D(2, 5, padding='same', activation='relu'),
            tf.keras.layers.Conv2D(1, 5, padding='same')
        ])

    def call(self, inputs, training=None, mask=None):
        common = self.common(inputs)
        value = self.value(common)
        policy_log = self.policy_log(common)
        return value, policy_log


model = Network()
model.compile()

input_signature = [tf.TensorSpec([None, 2, 8, 7], tf.float32, name="x")]
onnx_model, o = tf2onnx.convert.from_keras(model, input_signature, opset=14)
onnx.save(onnx_model, "test.tf.onnx.pb")
(5, 2, 8, 7)
x = tf.ones((5, 2, 8, 7))
print(x)

print(model.predict(x))
y = model.call(x)
print(y)