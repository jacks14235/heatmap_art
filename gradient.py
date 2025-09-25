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
    
    def __reversed__(self):
        colors = self.colors[::-1]
        stops = [1 - stop for stop in self.stops[::-1]]
        return Gradient(colors, stops[1:-1])

    def eval_np(self, n, weight=lambda x: np.ones_like(x)):
        # weights scales all points by a function of n at the end
        n_arr = np.atleast_1d(n).astype(np.float64)
        # clip to valid range [0, 1]
        n_clipped = np.clip(n_arr, 0.0, 1.0)

        stops = np.array(self.stops, dtype=np.float64)
        colors = np.array(self.colors, dtype=np.float64)

        # find interval indices (left/right) for each n
        indices = np.searchsorted(stops, n_clipped, side='left')
        left_idx = np.clip(indices - 1, 0, len(stops) - 1)
        right_idx = np.clip(indices, 0, len(stops) - 1)

        actual_left = stops[left_idx]
        actual_right = stops[right_idx]

        denom = actual_right - actual_left
        # avoid division by zero; where denom==0 keep dist=0
        dist = np.zeros_like(n_clipped, dtype=np.float64)
        nonzero = denom != 0
        dist[nonzero] = (n_clipped[nonzero] - actual_left[nonzero]) / denom[nonzero]

        # expand dist to RGB channels
        dist_rgb = np.repeat(dist, 3, axis=-1).reshape((*n_clipped.shape, 3))

        colors_left = colors[left_idx]
        colors_right = colors[right_idx]

        final = colors_left * (1 - dist_rgb) + colors_right * dist_rgb

        weights = weight(n_clipped)
        weights = np.expand_dims(weights, axis=-1).repeat(3, axis=-1)
        final = final * weights

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
        self.colors = self.colors[::-1]
        self.stops = [1 - stop for stop in self.stops[::-1]]
    
    def reflect(self):
        reversed_colors = self.colors[::-1]
        reversed_stops = [1 - stop for stop in self.stops[::-1]]
        self.colors += reversed_colors[1:]
        self.stops += [0.5 + stop * 0.5 for stop in reversed_stops[1:]]
        return self

    def sample(self, width=512, height=64):
        line = np.linspace(0, 1, 512)
        line = self.eval_np(line)
        print(line.shape)
        return np.repeat(line[np.newaxis, :, :], height, axis=0)
    
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
    
    def rainbow_no_wrap():
        return Gradient([
            [255,0,0],
            [255,255,0],
            [0,255,0],
            [0,255,255],
            [0,0,255],
            [255,0,255]
        ])

def main():
    gradient = Gradient(
        colors=[(255,0,0), (0,255,0), (0,0,255)]
    )


if __name__ == '__main__':
    main()

    