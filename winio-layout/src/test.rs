use winio_primitive::{Failable, HAlign, Orient, Point, Rect, Size, VAlign};

use crate::{Grid, Layoutable, StackPanel, layout};

#[derive(Debug, Clone)]
struct MockChild {
    preferred_size: Size,
    min_size: Size,
    loc: Point,
    size: Size,
}

impl MockChild {
    pub fn new(preferred_size: Size, min_size: Size) -> Self {
        Self {
            preferred_size,
            min_size,
            loc: Point::zero(),
            size: Size::zero(),
        }
    }

    #[track_caller]
    pub fn assert_loc(&self, x: f64, y: f64) {
        assert_eq!(self.loc, Point::new(x, y))
    }

    #[track_caller]
    pub fn assert_size(&self, width: f64, height: f64) {
        assert_eq!(self.size, Size::new(width, height))
    }
}

impl Failable for MockChild {
    type Error = ();
}

impl Layoutable for MockChild {
    fn loc(&self) -> Result<Point, ()> {
        Ok(self.loc)
    }

    fn set_loc(&mut self, p: Point) -> Result<(), ()> {
        self.loc = p;
        Ok(())
    }

    fn size(&self) -> Result<Size, ()> {
        Ok(self.size)
    }

    fn set_size(&mut self, s: Size) -> Result<(), ()> {
        self.size = s;
        Ok(())
    }

    fn preferred_size(&self) -> Result<Size, ()> {
        Ok(self.preferred_size)
    }

    fn min_size(&self) -> Result<Size, ()> {
        Ok(self.min_size)
    }
}

#[test]
fn stack_panel_horizontal() {
    let mut c1 = MockChild::new(Size::new(200.0, 100.0), Size::zero());
    let mut c2 = MockChild::new(Size::new(50.0, 50.0), Size::zero());
    let mut c3 = MockChild::new(Size::new(10.0, 200.0), Size::zero());

    let mut panel = layout! {
        StackPanel::new(Orient::Horizontal),
        c1, c2, c3
    };
    panel
        .set_rect(Rect::new(Point::zero(), Size::new(300.0, 200.0)))
        .unwrap();
    c1.assert_loc(0.0, 0.0);
    c2.assert_loc(200.0, 0.0);
    c3.assert_loc(250.0, 0.0);
    c1.assert_size(200.0, 200.0);
    c2.assert_size(50.0, 200.0);
    c3.assert_size(10.0, 200.0);

    let mut panel = layout! {
        StackPanel::new(Orient::Horizontal),
        c1, c2, c3
    };
    panel
        .set_rect(Rect::new(Point::zero(), Size::new(300.0, 10.0)))
        .unwrap();
    c1.assert_loc(0.0, 0.0);
    c2.assert_loc(200.0, 0.0);
    c3.assert_loc(250.0, 0.0);
    c1.assert_size(200.0, 10.0);
    c2.assert_size(50.0, 10.0);
    c3.assert_size(10.0, 10.0);

    let mut panel = layout! {
        StackPanel::new(Orient::Horizontal),
        c1, c2, c3 => { grow: true }
    };
    panel
        .set_rect(Rect::new(Point::zero(), Size::new(300.0, 200.0)))
        .unwrap();
    c1.assert_loc(0.0, 0.0);
    c2.assert_loc(200.0, 0.0);
    c3.assert_loc(250.0, 0.0);
    c1.assert_size(200.0, 200.0);
    c2.assert_size(50.0, 200.0);
    c3.assert_size(50.0, 200.0);

    let mut panel = layout! {
        StackPanel::new(Orient::Horizontal),
        c1, c2, c3 => { grow: true }
    };
    panel
        .set_rect(Rect::new(Point::zero(), Size::new(300.0, 10.0)))
        .unwrap();
    c1.assert_loc(0.0, 0.0);
    c2.assert_loc(200.0, 0.0);
    c3.assert_loc(250.0, 0.0);
    c1.assert_size(200.0, 10.0);
    c2.assert_size(50.0, 10.0);
    c3.assert_size(50.0, 10.0);

    let mut panel = layout! {
        StackPanel::new(Orient::Horizontal),
        c1 => { valign: VAlign::Top },
        c2 => { valign: VAlign::Center },
        c3 => { valign: VAlign::Bottom },
    };
    panel
        .set_rect(Rect::new(Point::zero(), Size::new(300.0, 400.0)))
        .unwrap();
    c1.assert_loc(0.0, 0.0);
    c2.assert_loc(200.0, 175.0);
    c3.assert_loc(250.0, 200.0);
    c1.assert_size(200.0, 100.0);
    c2.assert_size(50.0, 50.0);
    c3.assert_size(10.0, 200.0);
}

#[test]
fn stack_panel_vertical() {
    let mut c1 = MockChild::new(Size::new(200.0, 100.0), Size::zero());
    let mut c2 = MockChild::new(Size::new(50.0, 50.0), Size::zero());
    let mut c3 = MockChild::new(Size::new(10.0, 200.0), Size::zero());

    let mut panel = layout! {
        StackPanel::new(Orient::Vertical),
        c1, c2, c3
    };
    panel
        .set_rect(Rect::new(Point::zero(), Size::new(200.0, 400.0)))
        .unwrap();
    c1.assert_loc(0.0, 0.0);
    c2.assert_loc(0.0, 100.0);
    c3.assert_loc(0.0, 150.0);
    c1.assert_size(200.0, 100.0);
    c2.assert_size(200.0, 50.0);
    c3.assert_size(200.0, 200.0);

    let mut panel = layout! {
        StackPanel::new(Orient::Vertical),
        c1, c2, c3
    };
    panel
        .set_rect(Rect::new(Point::zero(), Size::new(10.0, 400.0)))
        .unwrap();
    c1.assert_loc(0.0, 0.0);
    c2.assert_loc(0.0, 100.0);
    c3.assert_loc(0.0, 150.0);
    c1.assert_size(10.0, 100.0);
    c2.assert_size(10.0, 50.0);
    c3.assert_size(10.0, 200.0);

    let mut panel = layout! {
        StackPanel::new(Orient::Vertical),
        c1, c2, c3 => { grow: true }
    };
    panel
        .set_rect(Rect::new(Point::zero(), Size::new(200.0, 400.0)))
        .unwrap();
    c1.assert_loc(0.0, 0.0);
    c2.assert_loc(0.0, 100.0);
    c3.assert_loc(0.0, 150.0);
    c1.assert_size(200.0, 100.0);
    c2.assert_size(200.0, 50.0);
    c3.assert_size(200.0, 250.0);

    let mut panel = layout! {
        StackPanel::new(Orient::Vertical),
        c1, c2, c3 => { grow: true }
    };
    panel
        .set_rect(Rect::new(Point::zero(), Size::new(10.0, 400.0)))
        .unwrap();
    c1.assert_loc(0.0, 0.0);
    c2.assert_loc(0.0, 100.0);
    c3.assert_loc(0.0, 150.0);
    c1.assert_size(10.0, 100.0);
    c2.assert_size(10.0, 50.0);
    c3.assert_size(10.0, 250.0);

    let mut panel = layout! {
        StackPanel::new(Orient::Vertical),
        c1 => { halign: HAlign::Left },
        c2 => { halign: HAlign::Center },
        c3 => { halign: HAlign::Right },
    };
    panel
        .set_rect(Rect::new(Point::zero(), Size::new(400.0, 400.0)))
        .unwrap();
    c1.assert_loc(0.0, 0.0);
    c2.assert_loc(175.0, 100.0);
    c3.assert_loc(390.0, 150.0);
    c1.assert_size(200.0, 100.0);
    c2.assert_size(50.0, 50.0);
    c3.assert_size(10.0, 200.0);
}

#[test]
fn grid_sudoku() {
    let mut c1 = MockChild::new(Size::new(50.0, 50.0), Size::zero());
    let mut c2 = c1.clone();
    let mut c3 = c1.clone();
    let mut c4 = c1.clone();
    let mut c5 = c1.clone();
    let mut c6 = c1.clone();
    let mut c7 = c1.clone();
    let mut c8 = c1.clone();
    let mut c9 = c1.clone();

    let mut grid = layout! {
        Grid::from_str("1*,1*,1*","1*,1*,1*").unwrap(),
        c1 => { column: 0, row: 0, halign: HAlign::Left,   valign: VAlign::Top },
        c2 => { column: 1, row: 0, halign: HAlign::Center, valign: VAlign::Top },
        c3 => { column: 2, row: 0, halign: HAlign::Right,  valign: VAlign::Top },
        c4 => { column: 0, row: 1, halign: HAlign::Left,   valign: VAlign::Center },
        c5 => { column: 1, row: 1, halign: HAlign::Center, valign: VAlign::Center },
        c6 => { column: 2, row: 1, halign: HAlign::Right,  valign: VAlign::Center },
        c7 => { column: 0, row: 2, halign: HAlign::Left,   valign: VAlign::Bottom },
        c8 => { column: 1, row: 2, halign: HAlign::Center, valign: VAlign::Bottom },
        c9 => { column: 2, row: 2, halign: HAlign::Right,  valign: VAlign::Bottom },
    };
    grid.set_rect(Rect::new(Point::zero(), Size::new(300.0, 300.0)))
        .unwrap();
    c1.assert_loc(0.0, 0.0);
    c2.assert_loc(125.0, 0.0);
    c3.assert_loc(250.0, 0.0);
    c4.assert_loc(0.0, 125.0);
    c5.assert_loc(125.0, 125.0);
    c6.assert_loc(250.0, 125.0);
    c7.assert_loc(0.0, 250.0);
    c8.assert_loc(125.0, 250.0);
    c9.assert_loc(250.0, 250.0);
    c1.assert_size(50.0, 50.0);
    c2.assert_size(50.0, 50.0);
    c3.assert_size(50.0, 50.0);
    c4.assert_size(50.0, 50.0);
    c5.assert_size(50.0, 50.0);
    c6.assert_size(50.0, 50.0);
    c7.assert_size(50.0, 50.0);
    c8.assert_size(50.0, 50.0);
    c9.assert_size(50.0, 50.0);
}
