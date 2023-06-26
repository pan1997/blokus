import tensorflow as tf

print(tf.__version__)


model = tf.keras.models.Sequential([
    tf.keras.layers.Flatten(input_shape=(2,3)),
    tf.keras.layers.Dense(5, activation='relu'),
    tf.keras.layers.Dense(2)
])

model.compile()

x = tf.constant([[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]])
print(x)


y = model.predict(x)
print(y)