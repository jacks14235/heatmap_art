import numpy as np

import numpy as np


class Gradient:
    def __init__(self, colors, stops=[]):
        if (len(stops) == 0):
            stops = [(i + 1) / (len(colors)-1) for i in range(len(colors)-2)]
        elif len(stops) != len(colors) - 2:
            print('legnth of stops must be two less than length of colors, 0 and 1 are assumed as endpoints')
            return None
        self.colors = colors
        stops.insert(0, 0)
        stops.append(1)
        self.stops = stops

    def eval_np(self, n):
        n = np.atleast_1d(n)
        n = np.clip(n, 0.0001,.9999)
        result = np.zeros((len(n), 3))

        indices = np.searchsorted(self.stops, n)
        # colors = np.array([*self.colors, self.colors[-1]])
        stops = np.array(self.stops)
        colors = np.array(self.colors)


        actual_left = stops[indices - 1]
        actual_right = stops[indices]

        dist = (n - actual_left) / (actual_right - actual_left)
        dist = np.repeat(dist, 3, axis=-1).reshape((*n.shape, 3))
        colors_left = colors[indices - 1]
        colors_right = colors[indices]
        final = colors_left * (1 - dist) + colors_right * dist
        
        return final.astype(np.uint8)

    def eval(self, n):
        try:
          if (n <= 0):
              return self.colors[0]
          if (n >= 1):
              return self.colors[-1]
          i = 0
          while self.stops[i] <= n:
              i += 1
          start = self.stops[i-1]
          end = self.stops[i]
          rat = (n - start) / (end - start)
          color = [0,0,0]
          for j in range(3):
            color[j] = (self.colors[i][j] - self.colors[i-1][j]) * rat + self.colors[i-1][j]
        except IndexError:
            print('Error', start, end, rat)
        return color
    
    def reverse(self):
        self.colors = [self.colors[-i] for i in range(1, len(self.colors) + 1)]
        self.stops = [(1 - self.stops[-i]) for i in range(1, len(self.stops) + 1)]
        return self
    
    def reflect(self):
        reversed_colors = self.colors[::-1]
        reversed_stops = [1 - stop for stop in self.stops[::-1]]
        self.colors += reversed_colors[1:]
        self.stops += [0.5 + stop * 0.5 for stop in reversed_stops[1:]]
        return self

    def sample(self, width=512, height=512):
        # Create empty image array
        img_array = np.zeros((width, height, 3), dtype=np.uint8)
        
        # Fill image: horizontal gradient
        for x in range(width):
            t = x / (width - 1)
            color = self.eval(t)
            img_array[:, x, :] = color  # fill column with the color
        
        # Convert to PIL and show/save
        return img_array
    
    def heat():
        return Gradient([
            [255,0,0],
            [255,255,0]
        ])

    def heat2():
        return Gradient([
            [255,0,0],
            [255,127,0],
            [255,255,0]
        ], [
            .66
        ])
    
    def cool():
        return Gradient([
            [0,0,255],
            [0,255,255],
            [0,255,0]
        ])
    
    def temp():
        return Gradient([
            [0,255,255],
            [0,255,0],
            [255,255,0],
            [255,0,0]
        ])
    
    def rainbow():
        return Gradient([
            [255,0,0],
            [255,255,0],
            [0,255,0],
            [0,255,255],
            [0,0,255],
            [255,0,255],
            [255,0,0]
        ])

def main():
    gradient = Gradient(
        colors=[(255,0,0), (0,255,0), (0,0,255)]
    )


if __name__ == '__main__':
    main()

    