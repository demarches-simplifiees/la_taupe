pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

pub struct Anchor {
    pub top_left: Point,
    pub bottom_right: Point,
    pub width: u32,
    pub height: u32,
}

impl Anchor {
    pub fn new(top_left: Point, bottom_right: Point) -> Self {
        Self {
            width: bottom_right.x - top_left.x,
            height: bottom_right.y - top_left.y,
            top_left,
            bottom_right,
        }
    }

    pub fn original_height(&self) -> u32 {
        self.bottom_right.y - self.top_left.y
    }

    pub fn iban_mask(&self) -> (u32, u32, u32, u32) {
        let x = self.top_left.x.saturating_sub(self.width);
        let y = self.top_left.y.saturating_sub(2 * self.height);

        let wrapping_width = (self.width * 2) * 7;
        let wrapping_height = self.height * 5;

        (x, y, wrapping_width, wrapping_height)
    }

    pub fn narrow_iban_mask(&self) -> (u32, u32, u32, u32) {
        let x = self.top_left.x.saturating_sub(self.width);
        let y = self
            .top_left
            .y
            .saturating_sub((0.5 * self.height as f32) as u32);

        let wrapping_width = (self.width * 4) * 7;
        let wrapping_height = self.height * 2;

        (x, y, wrapping_width, wrapping_height)
    }

    pub fn addr_mask(&self) -> (u32, u32, u32, u32) {
        let x = self
            .top_left
            .x
            .saturating_sub((self.width as f32 * 0.5) as u32);
        let y = self
            .top_left
            .y
            .saturating_sub((self.height as f32 * 7.5) as u32);

        let wrapping_width = self.width * 10;
        let wrapping_height = self.height * 9;

        (x, y, wrapping_width, wrapping_height)
    }

    pub fn right_align_addr_mask(&self) -> (u32, u32, u32, u32) {
        let x = self
            .top_left
            .x
            .saturating_sub((self.width as f32 * 5.0) as u32);
        let y = self
            .top_left
            .y
            .saturating_sub((self.height as f32 * 7.5) as u32);

        let wrapping_width = self.width * 10;
        let wrapping_height = self.height * 9;

        (x, y, wrapping_width, wrapping_height)
    }

    pub fn titulaire_mask(&self) -> (u32, u32, u32, u32) {
        let x = self
            .top_left
            .x
            .saturating_sub((self.width as f32 * 0.5) as u32);
        let y = self
            .top_left
            .y
            .saturating_sub((self.height as f32 * 0.5) as u32);

        let wrapping_width = self.width * 10;
        let wrapping_height = self.height * 4;

        (x, y, wrapping_width, wrapping_height)
    }
}
