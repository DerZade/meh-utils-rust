use geo::contains::Contains;
use geo::{CoordFloat, GeoFloat, Geometry, GeoNum, Line, Point, Rect};
use geo::algorithm::line_intersection::{line_intersection, LineIntersection};
use geo::map_coords::MapCoords;

#[cfg(test)]
mod tests {
    use geo::{Coordinate, Geometry, Line, Point, Rect};
    use crate::mvt::clip_feature::Clip;

    #[test]
    fn clip_point_returns_none_if_point_outside_of_box() {
        let rect = Rect::new(
            Coordinate {x: 0, y: 0},
            Coordinate {x: 5, y: 10},
        );
        let point: Geometry<i32> = geo::Geometry::Point(Point(Coordinate {x: 6, y: 5}));

        let clipped  = point.clip(&rect);

        assert!(clipped.is_none());
    }

    #[test]
    fn clip_point_returns_point_if_point_inside_of_box() {
        let rect = Rect::new(
            Coordinate {x: 0, y: 0},
            Coordinate {x: 5, y: 10},
        );
        let point: Geometry<i32> = geo::Geometry::Point(Point(Coordinate {x: 1, y: 5}));

        let clipped = point.clip(&rect);

        assert!(clipped.is_some());
        assert_eq!(clipped.unwrap(), point);
    }

    #[test]
    fn clip_line_returns_none_if_linestring_outside_of_box() {
        let rect = Rect::new(
            Coordinate {x: 0, y: 0},
            Coordinate {x: 5, y: 10},
        );

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: 4, y: -3},
            Coordinate {x: 8, y: 5},
        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_none());
    }

    #[test]
    fn clip_line_returns_complete_line_if_line_inside_of_box() {
        let rect = Rect::new(
            Coordinate {x: 0, y: 0},
            Coordinate {x: 5, y: 10},
        );

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: 1, y: 1},
            Coordinate {x: 3, y: 3},
        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_some());
        assert_eq!(clipped.unwrap(), line);
    }

    #[test]
    fn clip_line_returns_complete_line_if_line_on_edge_of_box() {
        let rect = Rect::new(
            Coordinate {x: 0, y: 0},
            Coordinate {x: 5, y: 10},
        );

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: 0, y: 0},
            Coordinate {x: 5, y: 0},
        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_some());
        assert_eq!(clipped.unwrap(), line);
    }
}

// it would be neat to generalize this to a Diff trait (subtract one geometry from another!)
// but that would be overkill much. do not need.
pub trait Clip<T: GeoFloat, Rhs=Self> {
    type Output;
    fn clip(&self, rect: &Rect<T>) ->  Option<Self::Output>;
}

impl<T: CoordFloat> Clip<T> for Geometry<T> {
    type Output=Geometry<T>;
    fn clip(&self, rect: &Rect<T>) -> Option<Geometry<T>> {
        match self {
            Geometry::Point(pt) => pt.clip(rect).map(|p| {Geometry::Point(p)}),
            Geometry::Line(l) => l.clip(rect).map(|l| {Geometry::Line(l)}),
            _ => None,
        }
    }
}

impl<T: GeoFloat> Clip<T> for Line<T> {
    type Output = Line<T>;

    fn clip(&self, rect: &Rect<T>) -> Option<Self::Output> {
        if rect.contains(&self.start) && rect.contains(&self.end) {
            Some(self.clone())
        } else {
            let box_lines = rect.to_polygon().exterior().lines().collect::<Vec<Line<T>>>();
             box_lines.into_iter().for_each(|box_line| {
                let intersection = line_intersection(box_line.clone(), self.map_coords(|(x, y)| {(x.to_f64(), y.to_f64())}));

            });


            let intersector = line_intersection(self, );
            // TODO
            None
        }
    }
}

impl<T: GeoFloat> Clip<T> for Point<T> {
    type Output = Point<T>;

    fn clip(&self, rect: &Rect<T>) -> Option<Self::Output> {
        rect.contains(self).then(|| {self.clone()})
    }
}