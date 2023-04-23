import numpy as np
import matplotlib.pyplot as plt
import sys

if len(sys.argv) != 2:
    exit("Provide one input file")

data = np.load(sys.argv[1])

fig, ax = plt.subplots(2)

# Z plot

for particle in data[:, :, 0].T:
    print(particle)
    ax[0].scatter(np.arange(0, data.shape[0]), particle)

ax[0].set_ylabel("Z (m)")
ax[0].set_xlabel("Component #")
ax[0].set_title("Time deviation")

# δ plot

for particle in data[:, :, 1].T:
    print(particle)
    ax[1].scatter(np.arange(0, data.shape[0]), particle)

ax[1].set_ylabel("δ (eV)")
ax[1].set_xlabel("Component #")
ax[1].set_title("Energy deviation")

fig.tight_layout()
plt.show()