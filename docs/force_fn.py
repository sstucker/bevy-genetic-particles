# -*- coding: utf-8 -*-

import matplotlib.pyplot as plt
import numpy as np


def force(x, repulsionRange, repulsionStrength, forceRange, forceStrength):
    if x > forceRange + repulsionRange:
        return 0
    else:
        if x > repulsionRange:
            return forceStrength * (1 - abs(repulsionRange + forceRange / 2 - x) / (forceRange / 2))
        else:
            return repulsionStrength * (repulsionRange - x) / repulsionRange
                

N = 1000

REPULSION_RANGE = 100
REPULSION_STRENGTH= 100

FORCE_RANGE = 500
FORCE_STRENGTH = -40

distances = range(N)[::-1]
s = [force(x, REPULSION_RANGE, REPULSION_STRENGTH, FORCE_RANGE, FORCE_STRENGTH) for x in distances]

plt.close(1)
plt.figure(1)
plt.plot(distances, s)
plt.ylim(-100, 100)
plt.xlim(0, N)

    