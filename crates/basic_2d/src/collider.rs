use std::ops::Add;

// A basic 2d box collider that can take any numeric type
// for coordinates.
#[derive(Debug)]
pub struct BoxCollider2d<S: PartialOrd> {
    left: S,
    right: S,
    top: S,
    bottom: S,
}

impl<S: PartialOrd + Copy> BoxCollider2d<S> {
    pub fn new(tlc: [S; 2], brc: [S; 2]) -> Self {
        Self {
            left: tlc[0],
            right: brc[0],
            top: tlc[1],
            bottom: brc[1],
        }
    }

    // Is a point located inside the collider?
    pub fn point_col(&self, point: [S; 2]) -> bool {
        let x_case = self.left <= point[0] && self.right >= point[0];
        let y_case = self.top >= point[1] && self.bottom <= point[1];
        return x_case && y_case;
    }

    // Is another box colliding with this box?
    // Will fail if self is fully consumed by other
    // i.e.:
    //          -------------------------
    //          |        other          |
    //          |     -------------     |
    //          |     |   self    |     |
    //          |     -------------     |
    //          |                       |
    //          -------------------------
    pub fn box_col(&self, other: &BoxCollider2d<S>) -> bool {
        let left_x = self.left < other.left && self.right > other.left;
        let right_x = self.left < other.right && self.right > other.right;
        let top_y = self.bottom < other.top && self.top > other.top;
        let bottom_y = self.bottom < other.bottom && self.top > other.bottom;
        (left_x || right_x) && (top_y || bottom_y)
    }
}

impl<S: PartialOrd + Copy + Add<S, Output = S>> BoxCollider2d<S> {
    // Move collider an amount in x and y directions
    pub fn translate(&mut self, dx: S, dy: S) {
        self.left = self.left + dx;
        self.right = self.right + dx;
        self.top = self.top + dy;
        self.bottom = self.bottom + dy;
    }
}
