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

INV_255 = 0.00392156862745098

def u8_to_range(uint8, minimum, maximum):
    return uint8 * (INV_255 * maximum - INV_255 * minimum) + minimum

y = [u8_to_range(i, 1, 11) for i in range(255)]

plt.close(2)
plt.figure(2)
plt.plot(range(255), y)

def range_to_u8(minimum, maximum, value):
    return (255. * (value - minimum)) / (maximum - minimum)
    
x = np.linspace(0.2, 0.3, 255)
y = [range_to_u8(0.2, 0.3, v) for v in x]
    
plt.close(3)
plt.figure(3)
plt.plot(x, y)
        