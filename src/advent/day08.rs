use crate::advent::AdventSolver;
use anyhow::{Error, format_err};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::ops;

#[derive(Default)]
pub struct Solver;

// From problem description (might be unique to me)
const WIDTH: usize = 25;
const HEIGHT: usize = 6;

#[derive(Default)]
struct ImageLayer {
    pub pixels: Vec<usize>,
    pub width: usize,
    pub height: usize,
}

#[derive(Default)]
struct Image {
    pub layers: Vec<ImageLayer>,
    pub width: usize,
    pub height: usize,
}

impl AdventSolver for Solver {
    fn solve(&mut self) -> Result<(), Error> {
        let mut image_data = String::new();
        File::open("input/day08.txt")?
             .read_to_string(&mut image_data)?;

        // Part 1: Find layer with fewest 0 digits
        let image = Image::from_string(WIDTH, HEIGHT, image_data.trim())?;
        let layer = image.layers
                         .iter()
                         .enumerate()
                         .map(|(i, l)| (i, l.count_pixels(0)))
                         .min_by_key(|&(_i, count)| count)
                         .unwrap().0;
        println!("Layer {} has fewest 0 pixels. Checksum: {}",
                 layer,
                 image.layers[layer].count_pixels(1) *
                 image.layers[layer].count_pixels(2));

        // Part 2: Render image
        print!("\n{}", image);

        Ok(())
    }
}

impl Image {
    fn empty(width: usize, height: usize) -> Self {
        Self {
            layers: Vec::new(),
            width: width,
            height: height,
        }
    }

    fn from_string(width: usize, height: usize,
                   image_data: &str) -> Result<Image, Error> {
        let ppl = width * height;
        if image_data.len() % ppl != 0 {
            return Err(format_err!("{} is not a multiple of {}",
                                   image_data.len(), ppl));
        }

        let mut image = Image::empty(width, height);
        let mut layer = ImageLayer::empty(width, height);
        for c in image_data.chars() {
            match c.to_digit(10) {
                Some(p) if p < 3 => layer.pixels.push(p as usize),
                _ => return Err(format_err!("Invalid char: {}", c)),
            }
            if layer.pixels.len() == ppl {
                image.layers.push(layer);
                layer = ImageLayer::empty(width, height);
            }
        }
        Ok(image)
    }

    fn flatten(&self) -> ImageLayer {
        let init = ImageLayer::transparent(self.width, self.height);
        self.layers.iter()
            .fold(init, |sum, layer| sum + layer)
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flat = self.flatten();
        for y in 0..flat.height {
            for x in 0..flat.width {
                let c = match flat.pixels[y*flat.width+x] {
                    0|2 => ' ',
                    1 => '\u{2588}',
                    _ => unreachable!(),
                };
                write!(f, "{}", c)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl ImageLayer {
    fn empty(width: usize, height: usize) -> Self {
        Self {
            pixels: Vec::new(),
            width: width,
            height: height,
        }
    }

    fn transparent(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![2; width*height],
            width: width,
            height: height,
        }
    }

    fn count_pixels(&self, value: usize) -> usize {
        self.pixels.iter()
                   .filter(|&&p| p == value)
                   .count()
    }
}

impl ops::Add<&ImageLayer> for ImageLayer {
    type Output = ImageLayer;

    fn add(self, rhs: &ImageLayer) -> ImageLayer {
        let mut result = ImageLayer::default();
        result.width = self.width;
        result.height = self.height;
        for i in 0..self.pixels.len() {
            let v = match self.pixels[i] {
                0 => 0,
                1 => 1,
                2 => rhs.pixels[i],
                _ => unreachable!(),
            };
            result.pixels.push(v);
        }
        result
    }
}
