#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Cube,
    Air,
}

impl Shape {
    pub fn is_visible(&self) -> bool {
        match self {
            Shape::Air => false,
            _ => true,
        }
    }

    pub fn is_solid(&self) -> bool {
        match self {
            Shape::Cube => true,
            Shape::Air => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Block {
    Air,
    Dirt,
    Grass,

    Unknown,
}

impl Block {
    pub fn from_id(id: u16) -> Self {
        match id {
            0 => Block::Air,
            1 => Block::Dirt,
            2 => Block::Grass,
            _ => Block::Unknown,
        }
    }

    pub fn shape(&self) -> Shape {
        match self {
            Block::Air => Shape::Air,
            _ => Shape::Cube,
        }
    }
}
