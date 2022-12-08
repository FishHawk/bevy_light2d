import matplotlib
import matplotlib.pyplot as plt
import numpy as np

image = plt.imread("a.png")
print(image.shape)

x = np.linspace(0, 1, 128)
y = np.power(np.e, 1.5 - 3 * x)
y = np.expand_dims(y, axis=1)
y = np.repeat(y, 2048, axis=1)

k = np.linspace(0, 1, 2048)
k = np.expand_dims(k, axis=0)
k = np.repeat(k, 128, axis=0)
y = np.power(k, y)

y = np.expand_dims(y, axis=2)
y = np.pad(y, ((0, 0), (0, 0), (0, 2)), "constant")
y = np.pad(y, ((0, 0), (0, 0), (0, 1)), "constant", constant_values=(1))
print(y.shape)

matplotlib.image.imsave("a1.png", y)
