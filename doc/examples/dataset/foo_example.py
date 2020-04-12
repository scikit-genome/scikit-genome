"""
Foo
===
"""
print(__doc__)

import matplotlib.pyplot
import numpy.random

image = numpy.random.random((224, 224, 3))

matplotlib.pyplot.figure(figsize=(3, 3))

matplotlib.pyplot.imshow(image)

matplotlib.pyplot.show()
