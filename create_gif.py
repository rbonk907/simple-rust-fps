import imageio.v3 as iio
from pathlib import Path
from pygifsicle import optimize

gif_path = "./output.gif"

images = list()
for file in Path("./output/").iterdir():
  if not file.is_file():
    continue

  images.append(iio.imread(file))

iio.imwrite(gif_path, images)
optimize(gif_path)