#[derive(Debug)]
pub enum Origin {
    Center(f32, f32),
    Corner(f32, f32),
}


/// DEM is short for Digital Elevation Model.
#[derive(Debug)]
pub struct DEMRaster {
    columns: usize,
    rows: usize,
    left: f32,
    bottom: f32,
    cell_size: f32,
    /// the magic value used for "unknown value in this cell"
    no_data_value: f32,
    /// ordered list of elevation raster values, to be folded into `columns` and `rows`
    data: Vec<f32>,
}

impl DEMRaster {
    pub fn new(
        columns: usize,
        rows: usize,
        origin: Origin,
        cell_size: f32,
        no_data_value: f32,
        data: Vec<f32>,
    ) -> Self {
        let (left, bottom) = match origin {
            Origin::Center(x, y) => (
                x - cell_size * (columns as f32) / 2.0,
                y - cell_size * (rows as f32) / 2.0,
            ),
            Origin::Corner(x, y) => (x, y),
        };

        DEMRaster {
            columns,
            rows,
            left,
            bottom,
            cell_size,
            no_data_value,
            data,
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.columns, self.rows)
    }

    pub fn x(&self, column: usize) -> f32 {
        self.left + column as f32 * self.cell_size
    }

    pub fn y(&self, row: usize) -> f32 {
        let norm_row = self.rows - row;
        self.bottom + norm_row as f32 * self.cell_size
    }

    pub fn z(&self, col: usize, row: usize) -> f32 {
        self.data[col + row * self.columns]
    }

    pub fn get_data(&self) -> &Vec<f32> {
        &self.data
    }

    pub fn get_no_data_value(&self) -> f32 {
        self.no_data_value
    }
}
