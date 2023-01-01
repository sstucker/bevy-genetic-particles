# -*- coding: utf-8 -*-
"""
Created on Wed Dec 28 11:30:39 2022

@author: sstucker
"""

import csv
import numpy as np
import matplotlib

cmap = matplotlib.cm.get_cmap('PuRd')
    
with open('cmap.csv', 'w') as f:
    writer = csv.writer(f)
    for x in np.linspace(0.0, 1.0, 255):
        writer.writerow(cmap(x))